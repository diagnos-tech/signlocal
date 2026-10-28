<script setup lang="ts">
import { installUrl, type SignResult } from "@signlocal/sdk";
import { errorText } from "@signlocal/sdk/messages";
import { computed, ref, shallowRef } from "vue";
import { useSignlocal } from "./useSignlocal";

const { status, signing, error, sign } = useSignlocal();
const text = ref("I agree to the terms.");
const result = shallowRef<SignResult>();

const setup = computed(() => errorText(status.value?.problem));
const failure = computed(
  () =>
    error.value &&
    (errorText(error.value) ?? { title: "Signing failed", body: error.value.message }),
);

async function onSign() {
  const data = new TextEncoder().encode(text.value);
  result.value = await sign({
    hash: "SHA-256",
    prepare: (_certificate, { hash }) => crypto.subtle.digest(hash, data),
  });
}
</script>

<template>
  <main>
    <h1>Sign a text (Vue)</h1>
    <p class="muted" role="status">
      {{ !status ? "Checking signlocal…" : setup ? `${setup.title}. ${setup.body}` : "signlocal is ready." }}
    </p>
    <a v-if="status?.problem === 'ExtensionMissing'" :href="installUrl()">Install signlocal</a>
    <label for="text">Text to sign</label>
    <textarea id="text" v-model="text" rows="3" style="width: 100%" />
    <p>
      <button type="button" :disabled="!status?.ready || signing" @click="onSign">
        {{ signing ? "Confirm in the signlocal window…" : "Sign with my certificate" }}
      </button>
    </p>
    <div aria-live="polite">
      <div v-if="failure" class="card error">
        <strong>{{ failure.title }}</strong>
        <p>{{ failure.body }}</p>
      </div>
      <div v-else-if="result && !signing" class="card">
        <p class="ok"><strong>Signed</strong> by {{ result.certificate.displayName }}</p>
        <p>{{ result.algorithm }} with {{ result.hash }}, {{ result.signature.length }}-byte signature.</p>
      </div>
    </div>
  </main>
</template>
