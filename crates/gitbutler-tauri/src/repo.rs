pub mod commands {
    use crate::error::{Error, UnmarkedError};
    use anyhow::{Context as _, Result};
    use but_graph::virtual_branches_legacy_types::BranchOwnershipClaims;
    use but_settings::AppSettingsWithDiskSync;
    use but_workspace::DiffSpec;
    use gitbutler_branch_actions::{hooks, RemoteBranchFile};
    use gitbutler_command_context::CommandContext;
    use gitbutler_oxidize::ObjectIdExt;
    use gitbutler_project::ProjectId;
    use gitbutler_repo::hooks::{HookResult, MessageHookResult};
    use gitbutler_repo::{FileInfo, RepoCommands};
    use std::path::Path;
    use std::sync::atomic::AtomicBool;
    use tauri::State;
    use tracing::instrument;

    #[tauri::command(async)]
    #[instrument(err(Debug))]
    pub fn git_get_local_config(id: ProjectId, key: &str) -> Result<Option<String>, Error> {
        let project = gitbutler_project::get(id)?;
        Ok(project.get_local_config(key)?)
    }

    #[tauri::command(async)]
    #[instrument(err(Debug))]
    pub fn git_set_local_config(id: ProjectId, key: &str, value: &str) -> Result<(), Error> {
        let project = gitbutler_project::get(id)?;
        project.set_local_config(key, value).map_err(Into::into)
    }

    #[tauri::command(async)]
    #[instrument(err(Debug))]
    pub fn check_signing_settings(id: ProjectId) -> Result<bool, Error> {
        let project = gitbutler_project::get(id)?;
        project.check_signing_settings().map_err(Into::into)
    }

    #[tauri::command(async)]
    #[instrument]
    pub fn git_clone_repository(
        repository_url: &str,
        target_dir: &Path,
    ) -> Result<(), UnmarkedError> {
        let should_interrupt = AtomicBool::new(false);

        gix::prepare_clone(repository_url, target_dir)?
            .fetch_then_checkout(gix::progress::Discard, &should_interrupt)
            .map(|(checkout, _outcome)| checkout)?
            .main_worktree(gix::progress::Discard, &should_interrupt)?;
        Ok(())
    }

    #[tauri::command(async)]
    #[instrument(skip(settings))]
    pub fn get_uncommited_files(
        settings: State<'_, AppSettingsWithDiskSync>,
        id: ProjectId,
    ) -> Result<Vec<RemoteBranchFile>, Error> {
        let project = gitbutler_project::get(id)?;

        let ctx = CommandContext::open(&project, settings.get()?.clone())?;
        Ok(gitbutler_branch_actions::get_uncommited_files(&ctx)?)
    }

    #[tauri::command(async)]
    #[instrument()]
    pub fn get_commit_file(
        project_id: ProjectId,
        relative_path: &Path,
        commit_id: String,
    ) -> Result<FileInfo, Error> {
        let project = gitbutler_project::get(project_id)?;
        let commit_id = git2::Oid::from_str(commit_id.as_ref()).map_err(anyhow::Error::from)?;
        Ok(project.read_file_from_commit(commit_id, relative_path)?)
    }

    #[tauri::command(async)]
    #[instrument()]
    pub fn get_workspace_file(
        project_id: ProjectId,
        relative_path: &Path,
    ) -> Result<FileInfo, Error> {
        let project = gitbutler_project::get(project_id)?;
        Ok(project.read_file_from_workspace(relative_path)?)
    }

    #[tauri::command(async)]
    #[instrument(skip(settings))]
    pub fn pre_commit_hook(
        settings: State<'_, AppSettingsWithDiskSync>,
        project_id: ProjectId,
        ownership: BranchOwnershipClaims,
    ) -> Result<HookResult, Error> {
        let project = gitbutler_project::get(project_id)?;
        let ctx = CommandContext::open(&project, settings.get()?.clone())?;
        let claim = ownership.into();
        Ok(hooks::pre_commit(&ctx, &claim)?)
    }

    #[tauri::command(async)]
    #[instrument(skip(settings))]
    pub fn pre_commit_hook_diffspecs(
        settings: State<'_, AppSettingsWithDiskSync>,
        project_id: ProjectId,
        changes: Vec<DiffSpec>,
    ) -> Result<HookResult, Error> {
        let project = gitbutler_project::get(project_id)?;
        let ctx = CommandContext::open(&project, settings.get()?.clone())?;

        let repository = ctx.gix_repo()?;
        let head = repository
            .head_tree_id_or_empty()
            .context("Failed to get head tree")?;

        let context_lines = settings.get()?.context_lines;

        let mut changes = changes.into_iter().map(Ok).collect::<Vec<_>>();

        let (new_tree, ..) = but_workspace::commit_engine::apply_worktree_changes(
            head.detach(),
            &repository,
            &mut changes,
            context_lines,
        )?;

        Ok(hooks::pre_commit_with_tree(&ctx, new_tree.to_git2())?)
    }

    #[tauri::command(async)]
    #[instrument(skip(settings))]
    pub fn post_commit_hook(
        settings: State<'_, AppSettingsWithDiskSync>,
        project_id: ProjectId,
    ) -> Result<HookResult, Error> {
        let project = gitbutler_project::get(project_id)?;
        let ctx = CommandContext::open(&project, settings.get()?.clone())?;
        Ok(gitbutler_repo::hooks::post_commit(&ctx)?)
    }

    #[tauri::command(async)]
    #[instrument(skip(settings))]
    pub fn message_hook(
        settings: State<'_, AppSettingsWithDiskSync>,
        project_id: ProjectId,
        message: String,
    ) -> Result<MessageHookResult, Error> {
        let project = gitbutler_project::get(project_id)?;
        let ctx = CommandContext::open(&project, settings.get()?.clone())?;
        Ok(gitbutler_repo::hooks::commit_msg(&ctx, message)?)
    }
}
