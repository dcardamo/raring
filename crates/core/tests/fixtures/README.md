All fixtures in this directory are synthetic. `health_status*.json` use
invented values with the metric keys and response shape checked by read-only
GraphQL calls; they are not captured account readings. They contain no Garmin account identifiers, health data, credentials, names, or coordinates from any real person. Field keys confirmed in python-garminconnect's typed.py and routes in __init__.py; guessed fields require a future manually reviewed, scrubbed capture.

`intensity_weeks.json` has invented values in the shape of
`/usersummary-service/stats/im/weekly/{start}/{end}` checked by a read-only
call: one row per Garmin intensity week, dated by its first day.
`exercise_sets*.json` follow the shape of `/activity-service/activity/{id}/exerciseSets`
verified read-only on a real account: set types, the
ranked exercise guesses with probabilities, weight in grams and the null
velocity fields. Exercises, reps, weights and times are invented.
`exercise_catalog.json` is a short excerpt in the shape of Garmin's public
catalog (`connect.garmin.com/web-data/exercises/Exercises.json`).
