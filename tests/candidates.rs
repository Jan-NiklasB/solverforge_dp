use dp::data::{generate, DemoData};
use dp::domain::ALL_QUALIFIKATIONEN;

#[test]
fn provided_kandidaten_are_used_when_valid() {
    let mut plan = generate(DemoData::Small);
    let nfs_index = plan
        .kandidaten
        .iter()
        .position(|candidate| candidate.qualifikation == ALL_QUALIFIKATIONEN[3])
        .unwrap();
    let chosen = *plan.kandidaten[nfs_index].mitarbeiter.last().unwrap();

    plan.kandidaten[nfs_index].mitarbeiter = vec![chosen];
    plan.prepare().unwrap();
    assert_eq!(plan.kandidaten[nfs_index].mitarbeiter, vec![chosen]);

    plan.kandidaten[nfs_index].mitarbeiter = vec![usize::MAX];
    plan.prepare().unwrap();
    assert!(plan.kandidaten[nfs_index].mitarbeiter.len() > 1);
}

#[test]
fn assignments_outside_candidates_are_unassigned() {
    let mut plan = generate(DemoData::Small);
    let nfs_index = plan
        .kandidaten
        .iter()
        .position(|candidate| candidate.qualifikation == ALL_QUALIFIKATIONEN[3])
        .unwrap();
    let employees = plan.kandidaten[nfs_index].mitarbeiter.clone();
    let chosen = employees[0];
    let outside = *employees.last().unwrap();
    plan.kandidaten[nfs_index].mitarbeiter = vec![chosen];

    let assignments: Vec<usize> = plan
        .schichtzuordnungs
        .iter()
        .enumerate()
        .filter(|(_, assignment)| assignment.qualifikation == ALL_QUALIFIKATIONEN[3])
        .map(|(index, _)| index)
        .collect();
    plan.schichtzuordnungs[assignments[0]].mitarbeiter_idx = Some(outside);
    plan.schichtzuordnungs[assignments[1]].mitarbeiter_idx = Some(chosen);

    plan.prepare().unwrap();

    assert_eq!(plan.schichtzuordnungs[assignments[0]].mitarbeiter_idx, None);
    assert_eq!(
        plan.schichtzuordnungs[assignments[1]].mitarbeiter_idx,
        Some(chosen)
    );
}

#[test]
fn unqualified_assignments_are_unassigned() {
    let mut plan = generate(DemoData::Small);
    let bad_by_qualification: Vec<Option<usize>> = ALL_QUALIFIKATIONEN
        .iter()
        .map(|qualification| {
            plan.mitarbeiters
                .iter()
                .find(|employee| !employee.qualifikation.contains(qualification))
                .map(|employee| employee.index)
        })
        .collect();
    for assignment in &mut plan.schichtzuordnungs {
        let qualification_index = ALL_QUALIFIKATIONEN
            .iter()
            .position(|qualification| *qualification == assignment.qualifikation)
            .unwrap();
        if let Some(employee_index) = bad_by_qualification[qualification_index] {
            assignment.mitarbeiter_idx = Some(employee_index);
        }
    }

    plan.prepare().unwrap();

    for assignment in &plan.schichtzuordnungs {
        let Some(employee_index) = assignment.mitarbeiter_idx else {
            continue;
        };
        let employee = &plan.mitarbeiters[employee_index];
        assert!(employee.qualifikation.contains(&assignment.qualifikation));
    }
}
