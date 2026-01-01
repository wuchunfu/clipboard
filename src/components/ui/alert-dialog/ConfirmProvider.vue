<script setup lang="ts">
import { confirmState, handleResult } from "@/composables/useConfirm";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
</script>

<template>
  <AlertDialog :open="confirmState.isOpen">
    <AlertDialogContent @escape-key-down.prevent @pointer-down-outside.prevent>
      <AlertDialogHeader>
        <AlertDialogTitle>{{ confirmState.title }}</AlertDialogTitle>
        <AlertDialogDescription v-if="confirmState.description">
          {{ confirmState.description }}
        </AlertDialogDescription>
      </AlertDialogHeader>

      <AlertDialogFooter>
        <AlertDialogCancel @click="handleResult(false)">
          {{ confirmState.cancelText }}
        </AlertDialogCancel>
        <AlertDialogAction
          @click="handleResult(true)"
          :class="
            confirmState.variant === 'destructive'
              ? 'bg-destructive text-destructive-foreground hover:bg-destructive/90'
              : ''
          "
        >
          {{ confirmState.actionText }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
