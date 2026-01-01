<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import MainWindow from "@/views/MainWindow.vue";
import PopupWindow from "@/views/PopupWindow.vue";
import ConfirmProvider from "./components/ui/alert-dialog/ConfirmProvider.vue";

const isPopup = ref(false);

onMounted(() => {
  const appWindow = getCurrentWindow();
  if (appWindow.label === "popup") {
    isPopup.value = true;
  }
});
</script>

<template>
  <PopupWindow v-if="isPopup" />
  <MainWindow v-else />
  <ConfirmProvider />
</template>
