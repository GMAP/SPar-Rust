#![feature(async_closure)]

use spar_rust::to_stream;

use criterion::{criterion_group, criterion_main, Criterion};
use futures::*;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use rust_spp::*;
use std::rc::Rc;

use futures::future::lazy;
struct ImageLine {
    line_index: usize,
    line_buffer: Vec<u8>,
}

fn render_line(size: usize, line: usize) -> Option<ImageLine> {
    let init_a = -2.125;
    let init_b = -1.5;
    let range = 3.0;
    let step = range / (size as f64);

    let mut m: Vec<u8> = vec![0; size];

    let i = line;

    let im = init_b + (step * (i as f64));
    let iterations = 10000;

    for (j, m) in m.iter_mut().enumerate() {
        let mut a = init_a + step * j as f64;
        let cr = a;

        let mut b = im;
        let mut k = 0;

        for ii in 0..iterations {
            let a2 = a * a;
            let b2 = b * b;
            if (a2 + b2) > 4.0 {
                break;
            }
            b = 2.0 * a * b + im;
            a = a2 - b2 + cr;
            k = ii;
        }
        *m = (255.0 - ((k as f64) * 255.0 / (iterations as f64))) as u8;
    }
    Some(ImageLine {
        line_index: line,
        line_buffer: m,
    })
}

struct ComputeLine {
    size: usize,
}
impl ComputeLine {
    fn new(size: usize) -> ComputeLine {
        ComputeLine { size }
    }
}
impl InOut<usize, ImageLine> for ComputeLine {
    fn process(&mut self, image_line: usize) -> Option<ImageLine> {
        render_line(self.size, image_line)
    }
}

struct RenderLine {}
impl In<ImageLine, ImageLine> for RenderLine {
    fn process(&mut self, image_line: ImageLine, _order: u64) -> ImageLine {
        image_line
    }
}

fn mandelbrot_sequential(size: usize) -> Vec<Option<ImageLine>> {
    (0..size)
        .map(|image_line| render_line(size, image_line))
        .collect()
}

fn mandelbrot_spar_rust(size: usize, threads: usize) {
    let lines: Vec<ImageLine> = to_stream!(OUTPUT(ImageLine), {
        for i in 0..size {
            STAGE(
                INPUT(size: usize, i: usize),
                OUTPUT(ImageLine),
                REPLICATE = threads,
                { render_line(size, i) },
            );
        }
    });

    let bytes: usize = lines.iter().map(|line| line.line_buffer.len()).sum();
    println!("Bytes: {bytes}")
}

fn mandelbrot_rustspp(size: usize, threads: usize) {
    let pipeline = pipeline![
        parallel!(
            move |line_index| render_line(size, line_index),
            threads as i32
        ),
        collect!()
    ];

    for i in 0..size {
        pipeline.post(i).unwrap();
    }
    let lines = pipeline.collect();
    let bytes: usize = lines.iter().map(|line| line.line_buffer.len()).sum();
    println!("Bytes: {bytes}")
}

fn mandelbrot_rustspp_ordered(size: usize, threads: usize) {
    let pipeline = pipeline![
        parallel!(ComputeLine::new(size), threads as i32),
        collect_ordered!()
    ];

    for i in 0..size {
        pipeline.post(i).unwrap();
    }
    let lines = pipeline.collect();
    let bytes: usize = lines.iter().map(|line| line.line_buffer.len()).sum();
    println!("Bytes: {bytes}")
}

#[tokio::main]
async fn mandelbrot_tokio(size: usize, threads: usize) {
    tokio_stream::iter(0..size)
        .map(move |index| {
            let (sender, receiver) = channel::oneshot::channel::<ImageLine>();
            tokio::spawn(lazy(move |_| {
                let result = render_line(size, index);
                sender.send(result.unwrap()).ok();
            }));
            receiver
        })
        .buffered(threads)
        .for_each(async move |_rendered_line| {})
        .await;
}

fn mandelbrot_rayon(size: usize, thread_pool: Rc<rayon::ThreadPool>) -> Vec<ImageLine> {
    let mut b = vec![];
    thread_pool.install(|| {
        (0..size)
            .into_par_iter()
            .map(|image_line| render_line(size, image_line).unwrap())
            .collect_into_vec(&mut b);
    });
    b
}

#[tokio::main]
async fn mandelbrot_tokio_unordered(size: usize, threads: usize) {
    tokio_stream::iter(0..size)
        .map(move |index| {
            let (sender, receiver) = channel::oneshot::channel::<ImageLine>();
            tokio::spawn(lazy(move |_| {
                let result = render_line(size, index);
                sender.send(result.unwrap()).ok();
            }));
            receiver
        })
        .buffer_unordered(threads)
        .for_each(async move |_rendered_line| {})
        .await;
}

fn mandelbrot_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("mandelbrot comparison");
    let threads_to_run = 4..=num_cpus::get();
    let routine = 1000;
    group.sample_size(10);
    for threads in threads_to_run {
        group.bench_with_input(
            &format!("spar_rust unordered {threads} worker threads"),
            &threads,
            |b, &threads| {
                b.iter(|| mandelbrot_spar_rust(routine, threads));
            },
        );
        group.bench_with_input(
            &format!("rust_ssp unordered {threads} worker threads"),
            &threads,
            |b, &threads| {
                b.iter(|| mandelbrot_rustspp(routine, threads));
            },
        );

        group.bench_with_input(
            &format!("rust_ssp ordered {threads} worker threads"),
            &threads,
            |b, &threads| {
                b.iter(|| mandelbrot_rustspp_ordered(routine, threads));
            },
        );

        group.bench_with_input(
            &format!("rayon {threads} worker threads"),
            &threads,
            |b, &threads| {
                let pool = Rc::new(
                    ThreadPoolBuilder::new()
                        .num_threads(threads)
                        .build()
                        .unwrap(),
                );
                b.iter(|| mandelbrot_rayon(routine, pool.clone()));
            },
        );

        group.bench_with_input(
            &format!("mandelbrot tokio ordered {threads} worker threads"),
            &threads,
            |b, &threads| {
                b.iter(|| mandelbrot_tokio(routine, threads));
            },
        );

        group.bench_with_input(
            &format!("mandelbrot tokio unordered {threads} worker threads"),
            &threads,
            |b, &threads| {
                b.iter(|| mandelbrot_tokio_unordered(routine, threads));
            },
        );
    }
}
//criterion_group!(benches, criterion_benchmark);
criterion_group!(benches, mandelbrot_benches);
criterion_main!(benches);
