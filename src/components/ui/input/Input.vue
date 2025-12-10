<script setup lang="ts">
import { cn } from "@/lib/utils";

defineProps<{
  modelValue: string | number;
  class?: any;
  type?: string;
}>();

defineEmits<{
  (e: "update:modelValue", value: string | number): void;
}>();

defineOptions({
  inheritAttrs: false,
});
</script>

<template>
  <div :class="cn('relative', $props.class)">
    <input
      v-bind="$attrs"
      :value="modelValue"
      @input="
        $emit('update:modelValue', ($event.target as HTMLInputElement).value)
      "
      :type="type || 'text'"
      class="flex h-full w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
    />
    <slot name="icon" />
  </div>
</template>
