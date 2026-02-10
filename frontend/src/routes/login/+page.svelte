<script lang="ts">
  import { enhance } from '$app/forms';
  import type { ActionData } from './$types';

  let { form }: { form: ActionData } = $props();
</script>

<div class="flex min-h-screen items-center justify-center bg-gray-50">
  <div class="w-full max-w-md space-y-8 rounded-lg bg-white p-8 shadow">
    <h2 class="text-center text-3xl font-bold text-gray-900">Sign in</h2>

    {#if form?.error}
      <div class="rounded border border-red-200 bg-red-50 px-4 py-3 text-red-700">
        {form.error}
      </div>
    {/if}

    <form method="POST" use:enhance>
      <div class="space-y-4">
        <div>
          <label for="username" class="block text-sm font-medium text-gray-700"> Username </label>
          <input
            type="text"
            name="username"
            id="username"
            required
            class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:ring-blue-500 focus:outline-none"
          />
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-gray-700"> Password </label>
          <input
            type="password"
            name="password"
            id="password"
            required
            class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:ring-blue-500 focus:outline-none"
          />
        </div>

        <button
          type="submit"
          class="flex w-full justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none"
        >
          Sign in
        </button>
      </div>
    </form>
  </div>
</div>
