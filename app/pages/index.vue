<template>
  <button @click="save_settings">
    Save Settings
  </button>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';

async function save_settings() {
  invoke("save_settings");
} 
</script>
