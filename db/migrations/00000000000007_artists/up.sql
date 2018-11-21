-- Define the artists table
CREATE TABLE artists (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  organization_id uuid NULL REFERENCES organizations (id),
  is_private BOOLEAN NOT NULL DEFAULT FALSE,
  name TEXT NOT NULL,
  bio TEXT NOT NULL,
  image_url TEXT NULL,
  thumb_image_url TEXT NULL,
  website_url TEXT NULL,
  youtube_video_urls TEXT[] DEFAULT '{}' NOT NULL,
  facebook_username TEXT,
  instagram_username TEXT,
  snapchat_username TEXT,
  soundcloud_username TEXT,
  bandcamp_username TEXT,
  spotify_id TEXT,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP DEFAULT now() NOT NULL
);

CREATE INDEX index_artists_name ON artists (name);
CREATE INDEX index_artists_organization_id ON artists (organization_id);