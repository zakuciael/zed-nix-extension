use std::fs;
use zed_extension_api::{self as zed, LanguageServerId, Result};

pub struct DeadnixLsp {
    cached_binary_path: Option<String>,
}

impl DeadnixLsp {
    pub const LANGUAGE_SERVER_ID: &'static str = "deadnix";

    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    pub fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let path = self.binary_path(language_server_id, worktree)?;
        Ok(zed::Command {
            command: path,
            args: vec![],
            env: worktree.shell_env(),
        })
    }

    fn binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("deadnix-ls") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|s| s.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "zed-extensions/nix",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();

        let target = match (arch, platform) {
            (zed::Architecture::X8664, zed::Os::Linux) => "x86_64-unknown-linux-gnu",
            (zed::Architecture::Aarch64, zed::Os::Linux) => "aarch64-unknown-linux-gnu",
            (zed::Architecture::X8664, zed::Os::Mac) => "x86_64-apple-darwin",
            (zed::Architecture::Aarch64, zed::Os::Mac) => "aarch64-apple-darwin",
            _ => {
                return Err(format!(
                    "deadnix-ls: unsupported platform ({arch:?}, {platform:?})"
                ))
            }
        };

        let asset_name = format!("deadnix-ls-{target}.tar.gz");

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("deadnix-ls: no release asset found for {asset_name:?}"))?;

        let version_dir = format!("deadnix-ls-{}", release.version);
        let binary_path = format!("{version_dir}/deadnix-ls");

        if !fs::metadata(&binary_path).is_ok_and(|s| s.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("deadnix-ls: failed to download: {e}"))?;

            zed::make_file_executable(&binary_path)
                .map_err(|e| format!("deadnix-ls: failed to make executable: {e}"))?;

            let entries = fs::read_dir(".")
                .map_err(|e| format!("deadnix-ls: failed to list directory: {e}"))?;
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_str() != Some(&version_dir)
                    && name.to_str().is_some_and(|n| n.starts_with("deadnix-ls-"))
                {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}
