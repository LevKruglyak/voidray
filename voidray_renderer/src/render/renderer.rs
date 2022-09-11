use std::{sync::{Arc, RwLock}, time::{Instant, Duration}, thread::{self, JoinHandle}};

use crossbeam::channel::{Sender, bounded};
use vulkano::device::Queue;

use crate::{scene::{Scene, Accelerable}, settings::{Settings}, vector::Float};

use super::{target::CpuRenderTarget, iterative::iterative_render};

pub struct RendererStats {
    currently_rendering: bool,
    samples: (u32, u32),
    time: Option<(Instant, Option<Instant>)>,
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
    pub fn one_shot(target: Arc<CpuRenderTarget>, stats: Arc<RwLock<RendererStats>>, scene: Arc<RwLock<Scene>>, settings: Arc<RwLock<Settings>>) -> Self {
        let (sender, receiver) = bounded(0);
        let thread_target = target.clone();
        let thread_scene = scene.clone();
        let thread_settings = settings.clone();
        let thread_stats = stats.clone();
    
        let handle = thread::spawn(move || {
            thread_stats.write().unwrap().currently_rendering = true;

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
            iterative_render(thread_target.clone(), &scene_accel, &settings.render, 1, total_samples);
            samples += 1;
            thread_stats.write().unwrap().samples = (samples, total_samples);

            // Calculate the appropriate update frequency
            let single_sample_time = Instant::now().duration_since(single_sample_time).as_secs_f64() as Float;
            let mut samples_per_frame = (settings.render.update_frequency / single_sample_time) as u32;
            samples_per_frame = std::cmp::min(std::cmp::max(samples_per_frame, 1), total_samples - samples);

            println!("decided to render {} samples per frame", samples_per_frame);
            
            while samples < total_samples {
                // Render
                let delta_samples = std::cmp::min(samples_per_frame, total_samples - samples);
                iterative_render(thread_target.clone(), &scene_accel, &settings.render, samples, total_samples);
                samples += delta_samples;
                thread_stats.write().unwrap().samples = (samples, total_samples);

                // Process other events
                if let Ok(action) = receiver.try_recv() {
                    match action {
                        RenderAction::Cancel => break,
                        _ => panic!("invalid action {:?}", action),
                    }
                }

                thread::sleep(Duration::from_millis(1));
            }

            // Finish rendering
            thread_stats.write().unwrap().currently_rendering = false;
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

    pub fn continuous(target: Arc<CpuRenderTarget>, stats: Arc<RwLock<RendererStats>>, scene: Arc<RwLock<Scene>>, settings: Arc<RwLock<Settings>>) -> Self {
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

                self.thread = Some(RenderThread::one_shot(self.target.clone(), self.stats.clone(), self.scene.clone(), self.settings.clone()));
            },
            RenderAction::Continuous => {
                if self.currently_rendering() {
                    panic!("invalid action {:?}", action);
                }

                self.thread = Some(RenderThread::continuous(self.target.clone(), self.stats.clone(), self.scene.clone(), self.settings.clone()));
            },
            action => {
                if let Some(thread) = &mut self.thread {
                    thread.execute(action);
                } else {
                    panic!("invalid action {:?}", action);
                }
            },
        }
    }

    pub fn currently_rendering(&self) -> bool {
        self.stats.read().unwrap().currently_rendering
    }

    pub fn samples(&self) -> (u32, u32) {
        self.stats.read().unwrap().samples
    }
}
