-- Add comment_count column to existing topics table
ALTER TABLE voicesphere.topics ADD comment_count int;

-- Update existing topics to have comment_count = 0
-- Note: In ScyllaDB, you need to update each row individually
-- This is a one-time migration script
