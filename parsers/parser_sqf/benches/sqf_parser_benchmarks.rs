use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tempfile::TempDir;
use hemtt_sqf::parser::{run as parse_sqf, database::Database};
use hemtt_workspace::{reporting::Processed, WorkspacePath};
use hemtt_preprocessor::Processor;

mod utils;
use utils::{
    generate_simple_assignments,
    generate_nested_arrays,
    generate_arsenal_init,
    generate_conditional_assignments,
    generate_variable_tracking,
    write_test_file,
    copy_benchmark_file_small,
    copy_benchmark_file_large
};

// Helper struct to hold pre-processed HEMTT context
struct BenchContext {
    database: Database,
    workspace_path: WorkspacePath,
}

impl BenchContext {
    fn new(file_path: &std::path::Path) -> Self {
        let database = Database::a3_with_workspace(&WorkspacePath::slim_file(file_path).unwrap(), false).unwrap();
        let workspace_path = WorkspacePath::slim_file(file_path).unwrap();
        Self { database, workspace_path }
    }

    fn create_processed(&self, content: String) -> Processed {
        // Create a temporary file with the content
        let temp_file = self.workspace_path.with_extension("sqf").unwrap();
        std::fs::write(temp_file.as_str(), content).unwrap();
        
        // Run the processor on the file
        Processor::run(&temp_file).unwrap()
    }
}

fn sqf_parser_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut group = c.benchmark_group("sqf_parser");
    
    // Configure to use 10 samples
    group.sample_size(10);
    
    // Benchmark: Real-world curated gear assignment functions
    {
        let small_path = copy_benchmark_file_small(&temp_dir);
        let large_path = copy_benchmark_file_large(&temp_dir);
        
        // Pre-create contexts outside benchmark
        let small_ctx = BenchContext::new(&small_path);
        let large_ctx = BenchContext::new(&large_path);
        
        let small_content = std::fs::read_to_string(&small_path).unwrap();
        let large_content = std::fs::read_to_string(&large_path).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("real_world", "small_benchmark"),
            &small_path,
            |b, _| {
                b.iter(|| {
                    let processed = small_ctx.create_processed(small_content.clone());
                    black_box(parse_sqf(&small_ctx.database, &processed).unwrap());
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("real_world", "large_benchmark"),
            &large_path,
            |b, _| {
                b.iter(|| {
                    let processed = large_ctx.create_processed(large_content.clone());
                    black_box(parse_sqf(&large_ctx.database, &processed).unwrap());
                });
            },
        );
    }
    
    // Benchmark 1: Simple assignments with increasing counts
    for count in [10, 50, 100, 500].iter() {
        let code = generate_simple_assignments(*count);
        let path = write_test_file(&temp_dir, &format!("simple_{count}.sqf"), &code);
        
        // Pre-create context
        let ctx = BenchContext::new(&path);
        
        group.bench_with_input(
            BenchmarkId::new("simple_assignments", count),
            count,
            |b, _| {
                b.iter(|| {
                    let processed = ctx.create_processed(code.clone());
                    black_box(parse_sqf(&ctx.database, &processed).unwrap());
                });
            },
        );
    }
    
    // Benchmark 2: Nested arrays with varying depth and width
    for (depth, width) in [(2, 2), (3, 3), (4, 2)].iter() {
        let code = generate_nested_arrays(*depth, *width);
        let path = write_test_file(
            &temp_dir,
            &format!("nested_d{depth}_w{width}.sqf"),
            &code
        );
        
        // Pre-create context
        let ctx = BenchContext::new(&path);
        
        group.bench_with_input(
            BenchmarkId::new("nested_arrays", format!("d{depth}w{width}")),
            &(*depth, *width),
            |b, _| {
                b.iter(|| {
                    let processed = ctx.create_processed(code.clone());
                    black_box(parse_sqf(&ctx.database, &processed).unwrap());
                });
            },
        );
    }
    
    // Benchmark 3: Arsenal initialization with increasing item counts
    for count in [100, 500, 1000, 5000].iter() {
        let code = generate_arsenal_init(*count);
        let path = write_test_file(&temp_dir, &format!("arsenal_{count}.sqf"), &code);
        
        // Pre-create context
        let ctx = BenchContext::new(&path);
        
        group.bench_with_input(
            BenchmarkId::new("arsenal_init", count),
            count,
            |b, _| {
                b.iter(|| {
                    let processed = ctx.create_processed(code.clone());
                    black_box(parse_sqf(&ctx.database, &processed).unwrap());
                });
            },
        );
    }
    
    // Benchmark 4: Conditional assignments with increasing depth
    for depth in [3, 5, 7, 10].iter() {
        let code = generate_conditional_assignments(*depth);
        let path = write_test_file(&temp_dir, &format!("conditional_{depth}.sqf"), &code);
        
        // Pre-create context
        let ctx = BenchContext::new(&path);
        
        group.bench_with_input(
            BenchmarkId::new("conditional_assignments", depth),
            depth,
            |b, _| {
                b.iter(|| {
                    let processed = ctx.create_processed(code.clone());
                    black_box(parse_sqf(&ctx.database, &processed).unwrap());
                });
            },
        );
    }
    
    // Benchmark 5: Variable tracking with different complexities
    for (vars, ops) in [(10, 10), (50, 20), (100, 50)].iter() {
        let code = generate_variable_tracking(*vars, *ops);
        let path = write_test_file(
            &temp_dir,
            &format!("variables_v{vars}_o{ops}.sqf"),
            &code
        );
        
        // Pre-create context
        let ctx = BenchContext::new(&path);
        
        group.bench_with_input(
            BenchmarkId::new("variable_tracking", format!("v{vars}o{ops}")),
            &(*vars, *ops),
            |b, _| {
                b.iter(|| {
                    let processed = ctx.create_processed(code.clone());
                    black_box(parse_sqf(&ctx.database, &processed).unwrap());
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, sqf_parser_benchmark);
criterion_main!(benches); 