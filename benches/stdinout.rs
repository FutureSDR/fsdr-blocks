use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fsdr_blocks::stdinout::{BytesOrder, StdOutSink};
use futuresdr::runtime::mocker::{Mocker, Reader};

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(unix)]
unsafe extern "C" {
    fn dup(fd: i32) -> i32;
    fn dup2(oldfd: i32, newfd: i32) -> i32;
    fn close(fd: i32) -> i32;
}

pub fn bench_stdinout(c: &mut Criterion) {
    let n_samp = 65536;
    let input: Vec<f32> = (0..n_samp).map(|x| x as f32).collect();

    let mut group = c.benchmark_group("stdinout");
    group.throughput(Throughput::Elements(n_samp as u64));

    // Redirect stdout to /dev/null during benchmark to prevent binary output flooding
    #[cfg(unix)]
    let null_file = std::fs::File::create("/dev/null").ok();
    #[cfg(unix)]
    let null_fd = null_file.as_ref().map(|f| f.as_raw_fd());
    #[cfg(unix)]
    let stdout_fd = unsafe { dup(1) };
    #[cfg(unix)]
    if let Some(fd) = null_fd {
        unsafe { dup2(fd, 1) };
    }

    group.bench_function("stdout_f32_64k", |b| {
        b.iter(|| {
            let block: StdOutSink<f32, Reader<f32>> = StdOutSink::new(BytesOrder::Native);
            let mut mocker = Mocker::new(block);
            mocker.input().set(input.clone());
            mocker.run();
        });
    });

    #[cfg(unix)]
    if stdout_fd >= 0 {
        unsafe {
            dup2(stdout_fd, 1);
            close(stdout_fd);
        }
    }

    group.finish();
}

criterion_group!(benches, bench_stdinout);
criterion_main!(benches);
