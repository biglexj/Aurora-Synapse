export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

export interface UpdateCheckResult {
  available: boolean;
  current_version: string;
  latest_version: string;
  release_name: string;
  release_notes: string;
  published_at: string;
  html_url: string;
  exe_url?: string;
  apk_url?: string;
  assets: ReleaseAsset[];
}
