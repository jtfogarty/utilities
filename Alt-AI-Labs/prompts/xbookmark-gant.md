```mermaid
gantt
    title X Bookmarks Export Process — Governor-Enforced Rate-Limited Gantt Chart
    dateFormat YYYY-MM-DD HH:mm
    axisFormat %H:%M
    todayMarker off

    section Batch 1
    Fetch + Store Batch 1 (~7 min)                  :b1fs, 2026-03-30 12:00, 15m
    Delete Batch 1 (800 deletes — governor 50 req / 15 min smooth pacing)  :b1del, after b1fs, 4h
    Force wait to next 15-min UTC window            :b1wait, after b1del, 15m

    section Batch 2
    Fetch + Store Batch 2 (~7 min)                  :b2fs, after b1wait, 15m
    Delete Batch 2 (800 deletes — governor 50 req / 15 min smooth pacing)  :b2del, after b2fs, 4h
    Force wait to next 15-min UTC window            :b2wait, after b2del, 15m

    section Batch 3
    Fetch + Store Batch 3 (~7 min)                  :b3fs, after b2wait, 15m
    Delete Batch 3 (800 deletes — governor 50 req / 15 min smooth pacing)  :b3del, after b3fs, 4h
    Force wait to next 15-min UTC window            :b3wait, after b3del, 15m

    section Notes
    Note: Fetch + Store combined for visibility (real duration ~7 min) :milestone, 2026-03-30 12:00, 0d
    --dry-run option available (skips deletes but respects full timing) :milestone, 2026-03-30 12:00, 0d
    ```