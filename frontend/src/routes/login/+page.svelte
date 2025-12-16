<script lang="ts">
    import { enhance } from '$app/forms';

    let error = '';
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50">
    <div class="max-w-md w-full space-y-8 p-8 bg-white rounded-lg shadow">
        <h2 class="text-3xl font-bold text-center text-gray-900">Sign in</h2>

        {#if error}
            <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
                {error}
            </div>
        {/if}

        <form method="POST" use:enhance>
            <div class="space-y-4">
                <div>
                    <label for="username" class="block text-sm font-medium text-gray-700">
                        Username
                    </label>
                    <input
                            type="text"
                            name="username"
                            id="username"
                            required
                            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                    />
                </div>

                <div>
                    <label for="password" class="block text-sm font-medium text-gray-700">
                        Password
                    </label>
                    <input
                            type="password"
                            name="password"
                            id="password"
                            required
                            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                    />
                </div>

                <button
                        type="submit"
                        class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                    Sign in
                </button>
            </div>
        </form>
    </div>
</div>
