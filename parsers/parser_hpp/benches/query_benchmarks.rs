use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parser_hpp::{HppClass, HppProperty, HppValue, DependencyExtractor};

fn create_simple_loadout() -> String {
    r#"class baseMan {
        displayName = "Unarmed";
        uniform[] = {"test_uniform"};
        vest[] = {"test_vest"};
        backpack[] = {"test_backpack"};
        primaryWeapon[] = {"test_rifle"};
        magazines[] = {"test_mag"};
    };"#.to_string()
}

fn create_deep_nested_loadout(depth: usize) -> String {
    let mut content = String::new();
    let mut current_class = String::from("baseMan");
    
    for i in 0..depth {
        content.push_str(&format!(r#"class {} {{
            displayName = "Level {}";
            uniform[] = {{"uniform_{}"}};
            primaryWeapon[] = {{"rifle_{}"}};"#,
            current_class, i, i, i
        ));
        current_class = format!("class_{}", i);
    }
    
    // Close all brackets
    for _ in 0..depth {
        content.push_str("\n};");
    }
    
    content
}

fn create_wide_loadout(width: usize) -> String {
    let mut content = String::new();
    
    for i in 0..width {
        content.push_str(&format!(r#"class soldier_{} {{
            displayName = "Soldier {}";
            uniform[] = {{"uniform_{}"}};
            vest[] = {{"vest_{}"}};
            primaryWeapon[] = {{"rifle_{}"}};
            magazines[] = {{"mag_{}"}};"#,
            i, i, i, i, i, i
        ));
        content.push_str("\n};");
    }
    
    content
}

fn create_mixed_loadout(depth: usize, width: usize) -> String {
    let mut content = String::new();
    let mut current_class = String::from("baseMan");
    
    // Create deep nesting first
    for i in 0..depth {
        content.push_str(&format!(r#"class {} {{
            displayName = "Level {}";
            "#,
            current_class, i
        ));
        
        // At each level, add width number of equipment
        for j in 0..width {
            content.push_str(&format!(r#"
                uniform_{j}[] = {{"uniform_{i}_{j}"}};
                vest_{j}[] = {{"vest_{i}_{j}"}};
                primaryWeapon_{j}[] = {{"rifle_{i}_{j}"}};"#,
                i=i, j=j
            ));
        }
        
        current_class = format!("class_{}", i);
    }
    
    // Close all brackets
    for _ in 0..depth {
        content.push_str("\n};");
    }
    
    content
}

fn benchmark_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("hpp_query_patterns");
    
    // Benchmark simple loadout
    let simple_loadout = create_simple_loadout();
    group.bench_function("simple_loadout", |b| {
        b.iter(|| {
            let parser = parser_hpp::HppParser::new(black_box(&simple_loadout)).unwrap();
            let classes = parser.parse_classes();
            let extractor = DependencyExtractor::new(classes);
            extractor.extract_dependencies()
        })
    });
    
    // Benchmark deeply nested loadout (depth=10)
    let deep_loadout = create_deep_nested_loadout(10);
    group.bench_function("deep_nested_loadout", |b| {
        b.iter(|| {
            let parser = parser_hpp::HppParser::new(black_box(&deep_loadout)).unwrap();
            let classes = parser.parse_classes();
            let extractor = DependencyExtractor::new(classes);
            extractor.extract_dependencies()
        })
    });
    
    // Benchmark wide loadout (width=100)
    let wide_loadout = create_wide_loadout(100);
    group.bench_function("wide_loadout", |b| {
        b.iter(|| {
            let parser = parser_hpp::HppParser::new(black_box(&wide_loadout)).unwrap();
            let classes = parser.parse_classes();
            let extractor = DependencyExtractor::new(classes);
            extractor.extract_dependencies()
        })
    });
    
    // Benchmark mixed loadout (depth=5, width=20)
    let mixed_loadout = create_mixed_loadout(5, 20);
    group.bench_function("mixed_loadout", |b| {
        b.iter(|| {
            let parser = parser_hpp::HppParser::new(black_box(&mixed_loadout)).unwrap();
            let classes = parser.parse_classes();
            let extractor = DependencyExtractor::new(classes);
            extractor.extract_dependencies()
        })
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_queries);
criterion_main!(benches); 