<template>
    <q-page padding>
        <div class="q-pa-md">
            <h3>Music</h3>
            <p>This is the music page.</p>
            <q-btn v-if="!musicStore.isPlaying" label="Play Sample Track" color="primary" @click="musicStore.play" />
            <q-btn v-if="musicStore.isPaused" label="Resume Track" color="primary" @click="musicStore.resume" />
            <q-btn v-else label="Pause Track" color="secondary" @click="musicStore.pause" />
        </div>
        <div class="q-pa-md">
            <h4>Current Directory Contents:</h4>
            <q-list bordered>
                <q-item v-for="(item, index) in musicStore.currentDirectoryContents" :key="index">
                    <q-item-section>{{ item }}</q-item-section>
                </q-item>
            </q-list>
        </div>
    </q-page>
</template>

<script setup lang="ts">
import { useMusicStore } from '../stores/musicStore';

const musicStore = useMusicStore();

musicStore.getFolderContents('');
</script>