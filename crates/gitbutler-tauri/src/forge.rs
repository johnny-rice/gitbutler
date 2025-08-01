pub mod commands {
    use std::path::Path;

    use anyhow::Context;
    use gitbutler_forge::{
        forge::ForgeName,
        review::{
            available_review_templates, get_review_template_functions, ReviewTemplateFunctions,
        },
    };
    use gitbutler_project::ProjectId;
    use gitbutler_repo::RepoCommands;
    use tracing::instrument;

    use crate::error::Error;

    #[tauri::command(async)]
    #[instrument(err(Debug))]
    pub fn pr_templates(project_id: ProjectId, forge: ForgeName) -> Result<Vec<String>, Error> {
        let project = gitbutler_project::get_validated(project_id)?;
        Ok(available_review_templates(&project.path, &forge))
    }

    #[tauri::command(async)]
    #[instrument()]
    pub fn pr_template(
        project_id: ProjectId,
        relative_path: &Path,
        forge: ForgeName,
    ) -> anyhow::Result<String, Error> {
        let project = gitbutler_project::get_validated(project_id)?;

        let ReviewTemplateFunctions {
            is_valid_review_template_path,
            ..
        } = get_review_template_functions(&forge);

        if !is_valid_review_template_path(relative_path) {
            return Err(anyhow::format_err!(
                "Invalid review template path: {:?}",
                Path::join(&project.path, relative_path)
            )
            .into());
        }
        Ok(project
            .read_file_from_workspace(relative_path)?
            .content
            .context("PR template was not valid UTF-8")?)
    }
}
