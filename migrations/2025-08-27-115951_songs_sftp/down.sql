-- Remove sftp_path column
ALTER TABLE songs DROP COLUMN sftp_path;

-- Add audio_url back
ALTER TABLE songs ADD COLUMN audio_url TEXT NOT NULL;
