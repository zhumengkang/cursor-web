use std::fs;
use std::path::Path;

fn main() {
    // 如果 node_bundle.tar.gz 不存在，创建空占位文件，避免编译报错
    // CI 构建时会在编译前生成真实的 bundle
    let bundle_path = Path::new("node_bundle.tar.gz");
    if !bundle_path.exists() {
        fs::write(bundle_path, b"").expect("创建 node_bundle.tar.gz 占位文件失败");
    }
    // 告知 cargo 当 bundle 变化时重新编译
    println!("cargo:rerun-if-changed=node_bundle.tar.gz");
}
