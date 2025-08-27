-- Remove audio_url if exists
ALTER TABLE songs DROP COLUMN audio_url;

-- Add sftp_path column
ALTER TABLE songs ADD COLUMN sftp_path VARCHAR(255) NOT NULL;
