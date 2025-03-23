use std::path::PathBuf;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mission_scanner::{
    scan_mission,
    types::{MissionScannerConfig, DEFAULT_FILE_EXTENSIONS, ReferenceType, MissionResults},
};
use tokio::runtime::Runtime;

#[derive(Debug)]
struct MissionMetrics {
    sqm_count: usize,
    sqf_count: usize,
    cpp_count: usize,
    total_files: usize,
    direct_deps: usize,
    inheritance_deps: usize,
    variable_deps: usize,
    total_deps: usize,
}

fn analyze_mission(results: &MissionResults) -> MissionMetrics {
    let sqm_count = results.sqm_file.iter().count();
    let sqf_count = results.sqf_files.len();
    let cpp_count = results.cpp_files.len();
    
    let (direct_deps, inheritance_deps, variable_deps) = results.class_dependencies
        .iter()
        .fold((0, 0, 0), |acc, dep| {
            match dep.reference_type {
                ReferenceType::Direct => (acc.0 + 1, acc.1, acc.2),
                ReferenceType::Inheritance => (acc.0, acc.1 + 1, acc.2),
                ReferenceType::Variable => (acc.0, acc.1, acc.2 + 1),
            }
        });

    MissionMetrics {
        sqm_count,
        sqf_count,
        cpp_count,
        total_files: sqm_count + sqf_count + cpp_count,
        direct_deps,
        inheritance_deps,
        variable_deps,
        total_deps: direct_deps + inheritance_deps + variable_deps,
    }
}

fn mission_scan_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = MissionScannerConfig {
        max_threads: num_cpus::get(),
        file_extensions: DEFAULT_FILE_EXTENSIONS.iter().map(|&s| s.to_string()).collect(),
    };

    let mut group = c.benchmark_group("mission_scanner");
    group.sample_size(10); // Reduced sample size due to I/O operations

    // Pre-analyze missions to get metrics
    let mission_paths = [
        ("simple", PathBuf::from("tests/fixtures/test_mission_1")),
        ("complex", PathBuf::from("tests/fixtures/test_mission_2")),
    ];

    let mission_metrics: Vec<(_, MissionMetrics)> = mission_paths
        .iter()
        .map(|(name, path)| {
            let results = rt.block_on(async {
                scan_mission(path, config.max_threads, &config).await.unwrap()
            });
            (*name, analyze_mission(&results))
        })
        .collect();

    // Print mission comparison info
    println!("\nMission Comparison:");
    for (name, metrics) in &mission_metrics {
        println!("\n{} Mission Metrics:", name);
        println!("Files: {} total ({} SQM, {} SQF, {} CPP)", 
            metrics.total_files, metrics.sqm_count, metrics.sqf_count, metrics.cpp_count);
        println!("Dependencies: {} total ({} direct, {} inheritance, {} variable)",
            metrics.total_deps, metrics.direct_deps, metrics.inheritance_deps, metrics.variable_deps);
    }
    println!();

    // Benchmark each file type separately for each mission
    for (name, path) in &mission_paths {
        // Full mission scan
        group.bench_function(format!("{}_mission/full_scan", name), |b| {
            b.iter(|| {
                rt.block_on(async {
                    black_box(
                        scan_mission(
                            path,
                            config.max_threads,
                            &config
                        ).await.unwrap()
                    )
                })
            });
        });

        // SQM-only scan
        let sqm_config = MissionScannerConfig {
            file_extensions: vec!["sqm".to_string()],
            ..config.clone()
        };
        group.bench_function(format!("{}_mission/sqm_only", name), |b| {
            b.iter(|| {
                rt.block_on(async {
                    black_box(
                        scan_mission(
                            path,
                            config.max_threads,
                            &sqm_config
                        ).await.unwrap()
                    )
                })
            });
        });

        // SQF-only scan
        let sqf_config = MissionScannerConfig {
            file_extensions: vec!["sqf".to_string()],
            ..config.clone()
        };
        group.bench_function(format!("{}_mission/sqf_only", name), |b| {
            b.iter(|| {
                rt.block_on(async {
                    black_box(
                        scan_mission(
                            path,
                            config.max_threads,
                            &sqf_config
                        ).await.unwrap()
                    )
                })
            });
        });

        // CPP/HPP-only scan
        let cpp_config = MissionScannerConfig {
            file_extensions: vec!["cpp".to_string(), "hpp".to_string()],
            ..config.clone()
        };
        group.bench_function(format!("{}_mission/cpp_only", name), |b| {
            b.iter(|| {
                rt.block_on(async {
                    black_box(
                        scan_mission(
                            path,
                            config.max_threads,
                            &cpp_config
                        ).await.unwrap()
                    )
                })
            });
        });
    }

    // Comparative throughput measurements
    for (name, metrics) in &mission_metrics {
        group.throughput(criterion::Throughput::Elements(metrics.total_files as u64));
        group.bench_with_input(
            BenchmarkId::new(format!("{}_mission/throughput", name), metrics.total_files),
            &metrics.total_files,
            |b, &_total_files| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            scan_mission(
                                &mission_paths.iter().find(|(n, _)| n == name).unwrap().1,
                                config.max_threads,
                                &config
                            ).await.unwrap()
                        )
                    })
                });
            }
        );
    }

    group.finish();
}

criterion_group!(benches, mission_scan_benchmark);
criterion_main!(benches); 