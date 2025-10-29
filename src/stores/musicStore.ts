import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { exists, readDir } from '@tauri-apps/plugin-fs';
import * as path from '@tauri-apps/api/path';



export const useMusicStore = defineStore('music', {
  state: () => ({
    rootDir: '/mnt/nas/Music',
    currentFolderPath: '',
    trackList: [] as string[],
    currentDirectoryContents: [] as string[],
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
    async getFolderContents(folderPath: string) {
      const fullPath = await path.join(this.rootDir, folderPath);
      readDir(fullPath)
        .then((entries) => {
          this.currentDirectoryContents = entries.map(entry => entry.name);
        })
        .catch((error) => {
          console.error('Error reading directory:', error);
        });
    }
  }
});