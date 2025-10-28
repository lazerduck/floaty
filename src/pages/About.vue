<template>
  <q-page padding>
    <div class="q-pa-md">
      <div class="q-mt-md">
        <q-btn color="primary" @click="runNeofetch" :loading="loading">
          Run neofetch
        </q-btn>
        <q-btn flat class="q-ml-sm" @click="output = ''">Clear</q-btn>
      </div>

      <div class="q-mt-md">
        <div v-if="error" class="text-negative">{{ error }}</div>
        <pre v-else class="neofetch-output" style="white-space: pre-wrap; background: #0b0b0b; color: #e6e6e6; padding: 12px; border-radius: 6px;">
{{ output || (loading ? 'Running...' : 'Click "Run neofetch" to show system info.') }}</pre>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from "@tauri-apps/api/core";

const output = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

async function runNeofetch() {
  loading.value = true
  error.value = null
  output.value = ''
  try {
    // Invoke the Rust command we added
    const res = await invoke<string>('run_neofetch')
    output.value = res
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  // Optionally run automatically on mount
  runNeofetch()
})
</script>

