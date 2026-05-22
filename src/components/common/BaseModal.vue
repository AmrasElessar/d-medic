<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue';
import { X } from 'lucide-vue-next';

interface Props {
  open: boolean;
  title?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closeOnBackdrop?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  closeOnBackdrop: true,
});

const emit = defineEmits<{
  (e: 'close'): void;
}>();

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) emit('close');
}

onMounted(() => window.addEventListener('keydown', onKey));
onUnmounted(() => window.removeEventListener('keydown', onKey));

watch(
  () => props.open,
  (v) => {
    document.body.style.overflow = v ? 'hidden' : '';
  },
);
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-150"
      enter-from-class="opacity-0"
      leave-active-class="transition duration-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="open"
        class="fixed inset-0 z-50 grid place-items-center bg-black/60 backdrop-blur-sm"
        @click.self="closeOnBackdrop && $emit('close')"
      >
        <div
          :class="[
            'bg-bg-elevated border border-border rounded-lg shadow-xl w-full',
            size === 'sm' && 'max-w-sm',
            size === 'md' && 'max-w-md',
            size === 'lg' && 'max-w-2xl',
            size === 'xl' && 'max-w-4xl',
          ]"
        >
          <header
            v-if="title || $slots.header"
            class="flex items-center justify-between px-5 py-3 border-b border-border"
          >
            <h2 class="text-base font-semibold">{{ title }}</h2>
            <button
              class="text-fg-muted hover:text-fg p-1 rounded"
              @click="$emit('close')"
            >
              <X class="w-4 h-4" />
            </button>
          </header>
          <div class="p-5">
            <slot />
          </div>
          <footer
            v-if="$slots.footer"
            class="flex justify-end gap-2 px-5 py-3 border-t border-border"
          >
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
