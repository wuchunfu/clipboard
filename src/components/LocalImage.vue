<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import { readFile } from "@tauri-apps/plugin-fs";

const props = defineProps<{
  src: string;
  alt?: string;
  class?: string;
}>();

const imageUrl = ref("");
const error = ref(false);

async function loadImage() {
  if (!props.src) {
    imageUrl.value = "";
    return;
  }

  // Revoke previous URL if exists
  if (imageUrl.value && imageUrl.value.startsWith("blob:")) {
    URL.revokeObjectURL(imageUrl.value);
  }

  // Check if it's a path or base64
  // If it starts with / or X:\, it's a path.
  // Otherwise assume base64 (legacy support or small images if logic changes)
  const isPath = props.src.startsWith("/") || props.src.match(/^[a-zA-Z]:\\/);

  if (!isPath) {
    // Assume base64
    imageUrl.value = `data:image/png;base64,${props.src}`;
    error.value = false;
    return;
  }

  try {
    const bytes = await readFile(props.src);
    const blob = new Blob([bytes]);
    imageUrl.value = URL.createObjectURL(blob);
    error.value = false;
  } catch (e) {
    console.error("Failed to load image:", props.src, e);
    error.value = true;
  }
}

watch(() => props.src, loadImage, { immediate: true });

onUnmounted(() => {
  if (imageUrl.value && imageUrl.value.startsWith("blob:")) {
    URL.revokeObjectURL(imageUrl.value);
  }
});
</script>

<template>
  <img
    v-if="!error && imageUrl"
    :src="imageUrl"
    :alt="alt"
    :class="props.class"
  />
  <div
    v-else
    :class="[
      props.class,
      'bg-muted/50 flex items-center justify-center text-muted-foreground',
    ]"
  >
    <span class="text-xs">Image Error</span>
  </div>
</template>
