<template>
    <q-page padding>
        <div class="q-pa-md">
            <h3>Music</h3>
            <p>This is the music page.</p>
            <q-btn v-if="!isPlaying" label="Play Sample Track" color="primary" @click="playSampleTrack" />
            <q-btn v-if="isPaused" label="Resume Track" color="primary" @click="resumeTrack" />
            <q-btn v-else label="Pause Track" color="secondary" @click="pauseTrack" />
        </div>
    </q-page>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const isPlaying = ref(false);
const isPaused = ref(false);

function playSampleTrack() {
    invoke('mpv_play', { path: '/mnt/nas/Music/Phase Zero - 位相零/Phase Zero - 位相零 - 01 Friday Night.flac' })
        .then(() => {
            console.log('Playing sample track');
        }).then(() => {
            isPlaying.value = true;
        })
        .catch((error) => {
            console.error('Error playing track:', error);
        });
}

function pauseTrack() {
    invoke('mpv_pause')
        .then(() => {
            console.log('Paused track');
        }).then(() => {
            isPaused.value = true;
        })
        .catch((error) => {
            console.error('Error pausing track:', error);
        });
}

function resumeTrack() {
    invoke('mpv_resume')
        .then(() => {
            console.log('Resumed track');
        }).then(() => {
            isPaused.value = false;
        })
        .catch((error) => {
            console.error('Error resuming track:', error);
        });
}
</script>