use gitbutler_branch::BranchCreateRequest;
use gitbutler_reference::Refname;

use super::*;

#[test]
fn unapply_with_data() {
    let Test {
        project,
        controller,
        repository,
        ..
    } = &Test::default();

    controller
        .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
        .unwrap();

    std::fs::write(repository.path().join("file.txt"), "content").unwrap();

    let (branches, _) = controller.list_virtual_branches(project).unwrap();
    assert_eq!(branches.len(), 1);

    controller
        .convert_to_real_branch(project, branches[0].id)
        .unwrap();

    assert!(!repository.path().join("file.txt").exists());

    let (branches, _) = controller.list_virtual_branches(project).unwrap();
    assert_eq!(branches.len(), 0);
}

#[test]
fn conflicting() {
    let Test {
        project,
        controller,
        repository,
        ..
    } = &Test::default();

    // make sure we have an undiscovered commit in the remote branch
    {
        fs::write(repository.path().join("file.txt"), "first").unwrap();
        let first_commit_oid = repository.commit_all("first");
        fs::write(repository.path().join("file.txt"), "second").unwrap();
        repository.commit_all("second");
        repository.push();
        repository.reset_hard(Some(first_commit_oid));
    }

    controller
        .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
        .unwrap();

    let unapplied_branch = {
        // make a conflicting branch, and stash it

        std::fs::write(repository.path().join("file.txt"), "conflict").unwrap();

        let (branches, _) = controller.list_virtual_branches(project).unwrap();
        assert_eq!(branches.len(), 1);
        assert!(branches[0].base_current);
        assert!(branches[0].active);
        assert_eq!(
            branches[0].files[0].hunks[0].diff,
            "@@ -1 +1 @@\n-first\n\\ No newline at end of file\n+conflict\n\\ No newline at end of file\n"
        );

        let unapplied_branch = controller
            .convert_to_real_branch(project, branches[0].id)
            .unwrap();

        Refname::from_str(&unapplied_branch).unwrap()
    };

    {
        // update base branch, causing conflict
        controller.update_base_branch(project).unwrap();

        assert_eq!(
            std::fs::read_to_string(repository.path().join("file.txt")).unwrap(),
            "second"
        );
    }

    let branch_id = {
        // apply branch, it should conflict
        let branch_id = controller
            .create_virtual_branch_from_branch(project, &unapplied_branch, None)
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(repository.path().join("file.txt")).unwrap(),
            "<<<<<<< ours\nconflict\n=======\nsecond\n>>>>>>> theirs\n"
        );

        let (branches, _) = controller.list_virtual_branches(project).unwrap();

        assert_eq!(branches.len(), 1);
        let branch = &branches[0];
        // assert!(!branch.base_current);
        assert!(branch.conflicted);
        assert_eq!(
            branch.files[0].hunks[0].diff,
            "@@ -1 +1,5 @@\n-first\n\\ No newline at end of file\n+<<<<<<< ours\n+conflict\n+=======\n+second\n+>>>>>>> theirs\n"
        );

        branch_id
    };

    {
        // Converting the branch to a real branch should put us back in an unconflicted state
        controller
            .convert_to_real_branch(project, branch_id)
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(repository.path().join("file.txt")).unwrap(),
            "second"
        );
    }
}

#[test]
fn delete_if_empty() {
    let Test {
        project,
        controller,
        ..
    } = &Test::default();

    controller
        .set_base_branch(project, &"refs/remotes/origin/master".parse().unwrap())
        .unwrap();

    controller
        .create_virtual_branch(project, &BranchCreateRequest::default())
        .unwrap();

    let (branches, _) = controller.list_virtual_branches(project).unwrap();
    assert_eq!(branches.len(), 1);

    controller
        .convert_to_real_branch(project, branches[0].id)
        .unwrap();

    let (branches, _) = controller.list_virtual_branches(project).unwrap();
    assert_eq!(branches.len(), 0);
}
