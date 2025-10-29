<template>
    <q-page padding>
        <div class="q-pa-md">
            <h3>Music</h3>
            <p>This is the music page.</p>

            <div class="row items-center q-gutter-sm">
                <q-btn dense round icon="skip_previous" aria-label="Previous" @click="prevTrack" />

                <q-btn dense round :icon="musicStore.isPlaying && !musicStore.isPaused ? 'pause' : 'play_arrow'" color="primary" aria-label="Play/Pause" @click="togglePlayPause" :disable="!musicStore.currentTrack" />

                <q-btn dense round icon="stop" aria-label="Stop" color="negative" @click="stopTrack" />

                <q-btn dense round icon="skip_next" aria-label="Next" @click="nextTrack" />

                <div class="q-ml-md text-subtitle2">{{ currentTrackDisplay }}</div>
            </div>
        </div>
        <div class="q-pa-md">
            <h4>Current Directory Contents:</h4>

            <div class="q-mb-sm row items-center">
                <q-btn dense flat round icon="arrow_back" :disable="!canGoUp" @click="goUp" />
                <div class="q-ml-sm">{{ displayPath }}</div>
            </div>

            <div style="max-height: 360px; overflow:auto; border:1px solid var(--q-color-grey-4); border-radius:6px; padding:6px;">
                <div class="q-mb-sm">
                    <q-input dense debounce="250" v-model="filter" placeholder="Filter files and folders" clearable />
                </div>
                <q-list bordered separator>
                    <q-item v-for="(item, index) in filteredContents" :key="index" clickable @click="onItemClick(item)">
                        <q-item-section side>
                            <q-icon :name="item.isDir ? 'folder' : 'music_note'" />
                        </q-item-section>
                        <q-item-section>
                            {{ item.name }}
                        </q-item-section>
                        <q-item-section side v-if="!item.isDir">
                            <q-btn dense flat icon="play_arrow" @click.stop="playFile(item)" />
                        </q-item-section>
                    </q-item>
                    <q-item v-if="filteredContents.length === 0">
                        <q-item-section>No items</q-item-section>
                    </q-item>
                </q-list>
            </div>
        </div>
    </q-page>
</template>

<script setup lang="ts">
import { useMusicStore } from '../stores/musicStore';
import { onMounted, computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const musicStore = useMusicStore();

const displayPath = computed(() => musicStore.currentFolderPath ? `/${musicStore.currentFolderPath}` : '/');
const canGoUp = computed(() => !!musicStore.currentFolderPath && musicStore.currentFolderPath !== '');

// filter input and filtered view
const filter = ref('');
const filteredContents = computed(() => {
    const q = filter.value.trim().toLowerCase();
    if (!q) return musicStore.currentDirectoryContents;
    return musicStore.currentDirectoryContents.filter(i => (i.name || '').toLowerCase().includes(q));
});

async function enterFolder(folderName: string) {
    const newPath = [musicStore.currentFolderPath, folderName].filter(Boolean).join('/');
    musicStore.currentFolderPath = newPath;
    await musicStore.getFolderContents(newPath);
}

const currentTrackDisplay = computed(() => musicStore.currentTrack || 'No track selected');

function togglePlayPause() {
    musicStore.togglePlayPause();
}

function stopTrack() {
    musicStore.stop();
}

async function prevTrack() {
    try {
        await musicStore.prevTrack();
    } catch (e) {
        console.warn('mpv_prev not available or failed', e);
    }
}

async function nextTrack() {
    try {
        await musicStore.nextTrack();
    } catch (e) {
        console.warn('mpv_next not available or failed', e);
    }
}

async function goUp() {
    if (!musicStore.currentFolderPath) return;
    const parts = musicStore.currentFolderPath.split('/').filter(Boolean);
    parts.pop();
    const newPath = parts.join('/');
    musicStore.currentFolderPath = newPath;
    await musicStore.getFolderContents(newPath);
}

function onItemClick(item: { name: string; isDir: boolean }) {
    if (item.isDir) {
        enterFolder(item.name);
    } else {
        // select the file as currentTrack but don't auto-play on click; user can press Play
        musicStore.currentTrack = item.name;
    }
}

function playFile(item: { name: string; isDir: boolean }) {
    if (item.isDir) return;
    musicStore.currentTrack = item.name;
    musicStore.play();
}

onMounted(() => {
    // initial load
    musicStore.getFolderContents(musicStore.currentFolderPath || '');
});
</script>