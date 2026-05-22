<script setup lang="ts">
import { CheckCircle2, AlertTriangle, Info, XCircle, X } from 'lucide-vue-next';
import { useToast } from '@/composables/useToast';

const toast = useToast();
</script>

<template>
  <div class="fixed top-4 right-4 z-40 flex flex-col gap-2 max-w-sm">
    <TransitionGroup
      enter-active-class="transition duration-200"
      enter-from-class="opacity-0 translate-x-4"
      leave-active-class="transition duration-150"
      leave-to-class="opacity-0 translate-x-4"
    >
      <div
        v-for="item in toast.state.items"
        :key="item.id"
        class="flex items-start gap-3 p-3 rounded-lg shadow-md bg-bg-elevated border"
        :class="{
          'border-accent/40': item.kind === 'info',
          'border-priority-low/40': item.kind === 'success',
          'border-priority-medium/40': item.kind === 'warning',
          'border-priority-critical/40': item.kind === 'error',
        }"
      >
        <Info v-if="item.kind === 'info'" class="w-5 h-5 text-accent shrink-0 mt-0.5" />
        <CheckCircle2 v-if="item.kind === 'success'" class="w-5 h-5 text-priority-low shrink-0 mt-0.5" />
        <AlertTriangle v-if="item.kind === 'warning'" class="w-5 h-5 text-priority-medium shrink-0 mt-0.5" />
        <XCircle v-if="item.kind === 'error'" class="w-5 h-5 text-priority-critical shrink-0 mt-0.5" />

        <div class="flex-1 min-w-0">
          <div class="text-sm font-medium text-fg">{{ item.title }}</div>
          <div v-if="item.body" class="text-xs text-fg-muted mt-0.5">{{ item.body }}</div>
        </div>

        <button
          class="text-fg-subtle hover:text-fg p-0.5 -mt-0.5 -mr-0.5"
          @click="toast.dismiss(item.id)"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
