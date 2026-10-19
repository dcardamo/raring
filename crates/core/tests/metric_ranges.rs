use raring_core::{store::Store,trends,view::{TrendBucket,TrendQuery,TrendRange}};
use serde_json::json;

#[test]
fn fourteen_day_range_has_daily_points_and_an_equal_comparison_window(){
 let store=Store::open(":memory:").unwrap();
 store.put("summary","2025-03-20","synthetic",&json!({"averageStressLevel":30})).unwrap();
 let query:TrendQuery=serde_json::from_value(json!({"metric":"stress","range":"days14","previous":true})).unwrap();
 let v=trends::trend(&store,&query,"2025-03-20").unwrap();
 assert_eq!(v.range,TrendRange::Days14);assert_eq!(v.bucket,TrendBucket::Day);
 assert_eq!(v.from,"2025-03-07");assert_eq!(v.to,"2025-03-20");assert_eq!(v.series.points.len(),14);
 let previous=v.previous.unwrap();assert_eq!(previous.points.first().unwrap().start,"2025-02-21");assert_eq!(previous.points.last().unwrap().end,"2025-03-06");
}
