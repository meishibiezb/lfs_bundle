use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, Context};
use regex::Regex;

fn main() -> Result<()> {
    pack_lfs_bundle()
}

fn pack_lfs_bundle() -> std::result::Result<(), anyhow::Error> {
    // 1. 配置参数 (对应脚本的变量)
    let start = "master~1";
    let end = "master";
    let bundle_file = "mybundle.bundle";
    let lfs_file = "mybundle_lfs.tar.gz";
    // Windows上习惯用 .tar.gz
    
    println!("--- Step 1: Creating Git Bundle ---");
    
    // 2. 调用 git bundle create
    // 命令: git bundle create mybundle.bundle master~1..master --
    let status = Command::new("git")
        .args(["bundle", "create", bundle_file, &format!("{}..{}", start, end), "--"])
        .status()
        .context("Failed to execute git bundle. Is Git installed and in PATH?")?;

    if !status.success() {
        anyhow::bail!("Git bundle creation failed.");
    }
    println!("Bundle created: {}", bundle_file);

    println!("--- Step 2: Identifying LFS Objects ---");

    // 3. 获取 LFS 文件列表
    // 命令: git lfs ls-files --long master~1 master
    let output = Command::new("git")
        .args(["lfs", "ls-files", "--long", start, end])
        .output()
        .context("Failed to execute git lfs ls-files")?;

    if !output.status.success() {
        // 如果没有 LFS 文件或者命令失败，这里简单警告一下，脚本可能本意也是允许无 LFS
        eprintln!("Warning: git lfs ls-files failed or returned no data.");
        return Ok(());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lfs_objects = parse_lfs_output(&output_str);

    if lfs_objects.is_empty() {
        println!("No LFS objects found in range.");
        return Ok(());
    }

    println!("Found {} LFS objects.", lfs_objects.len());

    // 4. 创建 Tar 包
    println!("--- Step 3: Creating LFS Archive ---");
    create_lfs_tar(lfs_file, &lfs_objects)?;

    println!("LFS Archive created: {}", lfs_file);
    println!("Done!");

    Ok(())
}

/// 解析 git lfs ls-files 的输出，提取 Hash 并转换为 Windows 路径
fn parse_lfs_output(output: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    // 正则匹配行首的 Hash: ([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]+)
    // 原脚本: sed -r 's_^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]+)_...'
    let re = Regex::new(r"^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]+)").unwrap();

    for line in output.lines() {
        if let Some(caps) = re.captures(line) {
            let p1 = &caps[1];
            let p2 = &caps[2];
            let rest = &caps[3];
            
            // 构建路径: .git/lfs/objects/p1/p2/p1p2rest
            // 在 Windows 上路径分隔符是 \
            let path = Path::new(".git")
                .join("lfs")
                .join("objects")
                .join(p1)
                .join(p2)
                .join(format!("{}{}{}", p1, p2, rest));
            
            paths.push(path);
        }
    }
    paths
}

/// 将文件列表打包进 Tar (替代 `tar --files-from=...`)
fn create_lfs_tar(filename: &str, files: &[PathBuf]) -> Result<()> {
    let file = File::create(filename)?;
    let gz_encoder = flate2::GzBuilder::new().write(file, flate2::Compression::default());
    let mut tar_builder = tar::Builder::new(gz_encoder);

    for path in files {
        if path.exists() {
            // 将文件添加到 tar 包中
            // append_path 会自动处理文件名
            tar_builder.append_path(path)?;
        } else {
            eprintln!("Warning: LFS object not found at {:?}", path);
        }
    }

    // 完成压缩
    let _ = tar_builder.into_inner()?;
    Ok(())
}