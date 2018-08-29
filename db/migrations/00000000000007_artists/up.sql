-- Define the artists table
CREATE TABLE artists (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL,
  bio TEXT NOT NULL,
  website_url TEXT NULL,
  youtube_video_urls TEXT[] DEFAULT '{}' NOT NULL,
  facebook_username TEXT,
  instagram_username TEXT,
  snapchat_username TEXT,
  soundcloud_username TEXT,
  bandcamp_username TEXT
);

CREATE INDEX index_artists_name ON artists (name);
