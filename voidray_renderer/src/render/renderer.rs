use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam::channel::{bounded, Sender};
use vulkano::device::Queue;

use crate::{
    scene::{Accelerable, Scene},
    settings::Settings,
    vector::Float,
};

use super::{iterative::iterative_render, target::CpuRenderTarget};

pub struct RendererStats {
    currently_rendering: bool,
    samples: (u32, u32),
    time: Option<(Instant, Option<Instant>)>,
    remaining: Option<Duration>,
}

pub struct RenderThread {
    handle: JoinHandle<()>,
    stats: Arc<RwLock<RendererStats>>,
    scene: Arc<RwLock<Scene>>,
    settings: Arc<RwLock<Settings>>,
    target: Arc<CpuRenderTarget>,
    sender: Sender<RenderAction>,
}

impl RenderThread {
    pub fn one_shot(
        target: Arc<CpuRenderTarget>,
        stats: Arc<RwLock<RendererStats>>,
        scene: Arc<RwLock<Scene>>,
        settings: Arc<RwLock<Settings>>,
    ) -> Self {
        let (sender, receiver) = bounded(0);
        let thread_target = target.clone();
        let thread_scene = scene.clone();
        let thread_settings = settings.clone();
        let thread_stats = stats.clone();

        let handle = thread::spawn(move || {
            thread_stats.write().unwrap().currently_rendering = true;

            // Start the timer
            let start_time = Instant::now();
            thread_stats.write().unwrap().time = Some((start_time, None));

            // Clear the target
            thread_target.clear();

            // Build the scene
            let scene_accel = thread_scene.read().unwrap().build_acceleration();
            let settings = thread_settings.read().unwrap().clone();

            // Begin rendering
            let mut samples = 0;
            let total_samples = settings.render.total_samples;

            // Time how long it takes to render a single sample
            let single_sample_time = Instant::now();
            iterative_render(thread_target.clone(), &scene_accel, &settings.render, 1);
            samples += 1;
            thread_stats.write().unwrap().samples = (samples, total_samples);

            // Calculate the appropriate update frequency
            let single_sample_time = Instant::now()
                .duration_since(single_sample_time)
                .as_secs_f64();
            thread_stats.write().unwrap().remaining = Some(Duration::from_secs_f64(
                single_sample_time * (total_samples - 1) as f64,
            ));
            let mut samples_per_frame =
                (settings.render.update_frequency / single_sample_time as Float) as u32;
            samples_per_frame =
                std::cmp::min(std::cmp::max(samples_per_frame, 1), total_samples - samples);

            // Main render loop
            while samples < total_samples {
                // Render
                let delta_samples = std::cmp::min(samples_per_frame, total_samples - samples);
                iterative_render(
                    thread_target.clone(),
                    &scene_accel,
                    &settings.render,
                    samples_per_frame,
                );
                samples += delta_samples;
                thread_stats.write().unwrap().samples = (samples, total_samples);
                let elapsed = Instant::now().duration_since(start_time).as_secs_f64();
                thread_stats.write().unwrap().remaining = Some(Duration::from_secs_f64(
                    elapsed / samples as f64 * (total_samples - samples) as f64,
                ));

                // Process other events
                if let Ok(action) = receiver.try_recv() {
                    match action {
                        RenderAction::Cancel => break,
                        _ => panic!("invalid action {:?}", action),
                    }
                }
            }

            // Finish rendering
            thread_stats.write().unwrap().currently_rendering = false;
            let time = thread_stats.read().unwrap().time;
            thread_stats.write().unwrap().time =
                time.map(|(start, _)| (start, Some(Instant::now())));
            thread_stats.write().unwrap().remaining = None;
        });

        Self {
            handle,
            stats,
            scene,
            settings,
            target,
            sender,
        }
    }

    pub fn continuous(
        target: Arc<CpuRenderTarget>,
        stats: Arc<RwLock<RendererStats>>,
        scene: Arc<RwLock<Scene>>,
        settings: Arc<RwLock<Settings>>,
    ) -> Self {
        let (sender, reciever) = bounded(0);
        let thread_target = target.clone();

        let handle = thread::spawn(move || {
            thread_target.clear();
        });

        Self {
            handle,
            stats,
            scene,
            settings,
            target,
            sender,
        }
    }

    pub fn execute(&mut self, action: RenderAction) {
        self.sender.send(action).unwrap();
    }
}

pub struct Renderer {
    queue: Arc<Queue>,
    stats: Arc<RwLock<RendererStats>>,
    scene: Arc<RwLock<Scene>>,
    settings: Arc<RwLock<Settings>>,
    target: Arc<CpuRenderTarget>,
    thread: Option<RenderThread>,
}

#[derive(Debug, PartialEq)]
pub enum RenderAction {
    // Single shot render
    Render,
    // Begin continually rendering
    Continuous,
    // Rebuild the scene and clear the screen
    Rebuild,
    // Cancel either single shot render of continuous render
    Cancel,
}

impl Renderer {
    pub fn new(
        queue: Arc<Queue>,
        scene: Arc<RwLock<Scene>>,
        settings: Arc<RwLock<Settings>>,
        target: Arc<CpuRenderTarget>,
    ) -> Self {
        let stats = RendererStats {
            currently_rendering: false,
            samples: (0, 0),
            time: None,
            remaining: None,
        };

        Self {
            queue,
            stats: Arc::new(RwLock::new(stats)),
            scene,
            settings,
            target,
            thread: None,
        }
    }

    pub fn execute(&mut self, action: RenderAction) {
        match action {
            RenderAction::Render => {
                if self.currently_rendering() {
                    panic!("invalid action {:?}", action);
                }

                self.thread = Some(RenderThread::one_shot(
                    self.target.clone(),
                    self.stats.clone(),
                    self.scene.clone(),
                    self.settings.clone(),
                ));
            }
            RenderAction::Continuous => {
                if self.currently_rendering() {
                    panic!("invalid action {:?}", action);
                }

                self.thread = Some(RenderThread::continuous(
                    self.target.clone(),
                    self.stats.clone(),
                    self.scene.clone(),
                    self.settings.clone(),
                ));
            }
            action => {
                if let Some(thread) = &mut self.thread {
                    thread.execute(action);
                } else {
                    panic!("invalid action {:?}", action);
                }
            }
        }
    }

    pub fn currently_rendering(&self) -> bool {
        self.stats.read().unwrap().currently_rendering
    }

    pub fn samples(&self) -> (u32, u32) {
        self.stats.read().unwrap().samples
    }

    pub fn elapsed_time(&self) -> Duration {
        let stats = self.stats.read().unwrap();

        if let Some((start, end)) = stats.time {
            #[allow(clippy::or_fun_call)]
            return end.unwrap_or(Instant::now()).duration_since(start);
        }

        Duration::default()
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        self.stats.read().unwrap().remaining
    }
}
