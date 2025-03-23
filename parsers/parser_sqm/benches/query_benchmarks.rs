use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hemtt_sqm::{Class, SqmFile, Value};
use parser_sqm::extract_class_dependencies;
use std::collections::HashMap;

fn create_simple_mission() -> String {
    r#"class Mission {
        class Item1 {
            class Attributes {
                class Inventory {
                    uniform = "test_uniform";
                    vest = "test_vest";
                    class primaryWeapon {
                        name = "test_rifle";
                        class primaryMuzzleMag {
                            name = "test_mag";
                        };
                    };
                };
            };
        };
    };"#.to_string()
}

fn create_deep_nested_mission(depth: usize) -> String {
    let mut content = String::from("class Mission {");
    let mut current = String::new();
    
    for i in 0..depth {
        current.push_str(&format!(r#"
            class Item{} {{
                class Attributes {{
                    class Inventory {{"#, i));
    }
    
    // Add some content at the deepest level
    current.push_str(r#"
                        uniform = "test_uniform";
                        class primaryWeapon {
                            name = "test_rifle";
                            class primaryMuzzleMag {
                                name = "test_mag";
                            };
                        };"#);
    
    // Close all brackets
    for _ in 0..depth {
        current.push_str("\n                    };\n                };\n            };");
    }
    
    content.push_str(&current);
    content.push_str("\n};");
    content
}

fn create_wide_mission(width: usize) -> String {
    let mut content = String::from("class Mission {");
    
    for i in 0..width {
        content.push_str(&format!(r#"
            class Item{} {{
                class Attributes {{
                    class Inventory {{
                        uniform = "test_uniform_{}";
                        vest = "test_vest_{}";
                        class primaryWeapon {{
                            name = "test_rifle_{}";
                            class primaryMuzzleMag {{
                                name = "test_mag_{}";
                            }};
                        }};
                    }};
                }};
            }};"#, i, i, i, i, i));
    }
    
    content.push_str("\n};");
    content
}

fn create_mixed_mission(depth: usize, width: usize) -> String {
    let mut content = String::from("class Mission {");
    let mut current = String::new();
    
    // Create deep nesting first
    for i in 0..depth {
        current.push_str(&format!(r#"
            class Deep{} {{
                class Attributes {{
                    class Inventory {{"#, i));
    }
    
    // At the deepest level, create wide structure
    for i in 0..width {
        current.push_str(&format!(r#"
                        class Item{} {{
                            uniform = "test_uniform_{}";
                            class primaryWeapon {{
                                name = "test_rifle_{}";
                            }};
                        }};"#, i, i, i));
    }
    
    // Close deep nesting
    for _ in 0..depth {
        current.push_str("\n                    };\n                };\n            };");
    }
    
    content.push_str(&current);
    content.push_str("\n};");
    content
}

fn benchmark_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_patterns");
    
    // Benchmark simple mission
    let simple_mission = create_simple_mission();
    group.bench_function("simple_mission", |b| {
        b.iter(|| extract_class_dependencies(black_box(&simple_mission)))
    });
    
    // Benchmark deeply nested mission (depth=10)
    let deep_mission = create_deep_nested_mission(10);
    group.bench_function("deep_nested_mission", |b| {
        b.iter(|| extract_class_dependencies(black_box(&deep_mission)))
    });
    
    // Benchmark wide mission (width=100)
    let wide_mission = create_wide_mission(100);
    group.bench_function("wide_mission", |b| {
        b.iter(|| extract_class_dependencies(black_box(&wide_mission)))
    });
    
    // Benchmark mixed mission (depth=5, width=20)
    let mixed_mission = create_mixed_mission(5, 20);
    group.bench_function("mixed_mission", |b| {
        b.iter(|| extract_class_dependencies(black_box(&mixed_mission)))
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_queries);
criterion_main!(benches); 