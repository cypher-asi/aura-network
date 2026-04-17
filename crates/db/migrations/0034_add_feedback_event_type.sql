-- Extend the activity_events event_type CHECK to include 'feedback' so the
-- Feedback app can persist posts through the existing activity_events table.

ALTER TABLE activity_events DROP CONSTRAINT IF EXISTS activity_events_event_type_check;
ALTER TABLE activity_events ADD CONSTRAINT activity_events_event_type_check
    CHECK (event_type IN (
        'commit',
        'task_completed',
        'task_failed',
        'loop_started',
        'loop_finished',
        'agent_created',
        'post',
        'push',
        'feedback'
    ));
