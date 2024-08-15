use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nmd_core::{loader::{loader_configuration::LoaderConfiguration, Loader}, output_format::OutputFormat};

fn load_dossier(dossier_path: &PathBuf) {

    // let codex = Codex::of_html(CodexConfiguration::default());

    // let loader_configuration = LoaderConfiguration::default();

    // let _dossier = Loader::load_dossier_from_path_buf(dossier_path, &codex, &loader_configuration).unwrap();

}

fn criterion_benchmark(c: &mut Criterion) {

    let dossier_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources").join("nmd-test-dossier-1");

    c.bench_function("load_dossier", |b| b.iter(|| load_dossier(black_box(&dossier_path))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);