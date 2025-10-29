import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { exists, readDir } from '@tauri-apps/plugin-fs';
import * as path from '@tauri-apps/api/path';



export const useMusicStore = defineStore('music', {
  state: () => ({
    rootDir: '/mnt/nas/Music',
    currentFolderPath: '',
    trackList: [] as string[],
    // store name + isDir so UI can distinguish folders from files
    currentDirectoryContents: [] as { name: string; isDir: boolean }[],
    currentTrack: '',
    isPlaying: false,
    isPaused: false,
  }),
  actions: {
    async play() {
      var fullPath = await path.join(this.rootDir, this.currentFolderPath, this.currentTrack);
      if (!(await exists(fullPath))) {
        console.error('Track does not exist:', fullPath);
        return;
      }
      invoke('mpv_play', { path: fullPath })
        .then(() => {
          this.isPlaying = true;
          this.isPaused = false;
        })
        .catch((error) => {
          console.error('Error playing track:', error);
        });
    },
    pause() {
      invoke('mpv_pause')
        .then(() => {
          this.isPlaying = true;
          this.isPaused = true;
        })
        .catch((error) => {
          console.error('Error pausing track:', error);
        });
    },
    resume() {
      invoke('mpv_resume')
        .then(() => {
          this.isPlaying = true;
          this.isPaused = false;
        })
        .catch((error) => {
          console.error('Error resuming track:', error);
        });
    },
    togglePlayPause() {
      if (this.isPaused) {
        this.resume();
      } else {
        this.pause();
      }
    },
    stop() {
      invoke('mpv_stop')
        .then(() => {
          this.isPlaying = false;
          this.isPaused = false;
        })
        .catch((error) => {
          console.error('Error stopping track:', error);
        });
    },
    async prevTrack() {
      const files = this.currentDirectoryContents.filter(f => !f.isDir).map(f => f.name);
      if (files.length === 0) return;
      let idx = files.indexOf(this.currentTrack);
      if (idx === -1) idx = 0;
      const newIdx = idx > 0 ? idx - 1 : files.length - 1;
      this.currentTrack = files[newIdx];
      await this.play();
    },
    async nextTrack() {
      const files = this.currentDirectoryContents.filter(f => !f.isDir).map(f => f.name);
      if (files.length === 0) return;
      let idx = files.indexOf(this.currentTrack);
      // if not found, start from before first so next moves to 0
      if (idx === -1) idx = -1;
      const newIdx = (idx + 1) % files.length;
      this.currentTrack = files[newIdx];
      await this.play();
    },
    async getFolderContents(folderPath: string) {
      const fullPath = await path.join(this.rootDir, folderPath);
      readDir(fullPath)
        .then((entries) => {
          this.currentDirectoryContents = entries.map(entry => ({
            name: entry.name ?? '',
            isDir: entry.isDirectory
          }));
        })
        .catch((error) => {
          console.error('Error reading directory:', error);
        });
    }
  }
});