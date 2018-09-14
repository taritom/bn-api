use bigneon_db::models::{FeeSchedule, NewFeeScheduleRange};
use support::project::TestProject;

#[test]
fn fee_schedule_create() {
    let project = TestProject::new();
    let fee_schedule = FeeSchedule::create(
        "default".to_string(),
        vec![
            NewFeeScheduleRange {
                min_price: 20,
                fee: 10,
            },
            NewFeeScheduleRange {
                min_price: 1000,
                fee: 100,
            },
        ],
    ).commit(project.get_connection())
    .unwrap();

    let ranges = fee_schedule.ranges(project.get_connection()).unwrap();
    assert_eq!(
        vec![ranges[0].min_price, ranges[1].min_price],
        vec![20, 1000]
    );
    assert_eq!(vec![ranges[0].fee, ranges[1].fee], vec![10, 100]);

    let fee_schedule2 = FeeSchedule::create(
        "default".to_string(),
        vec![
            NewFeeScheduleRange {
                min_price: 20,
                fee: 10,
            },
            NewFeeScheduleRange {
                min_price: 1000,
                fee: 100,
            },
        ],
    ).commit(project.get_connection())
    .unwrap();

    assert_eq!(fee_schedule2.version, 1);
}
