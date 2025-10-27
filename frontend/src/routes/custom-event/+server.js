// src/routes/custom-event/+server.js
import { produce } from 'sveltekit-sse'

export function POST() {
    return produce(async function start({ emit }) {
        while (true) {
            let reader = null;

            try {
                const response = await fetch("http://localhost:3033/sse");

                if (!response.ok) {
                    console.error("Failed to fetch SSE:", response.statusText);
                    await delay(5000);
                    continue;
                }

                reader = response.body?.getReader();
                const decoder = new TextDecoder();

                if (!reader) {
                    console.error("No reader available");
                    await delay(5000);
                    continue;
                }

                let buffer = '';

                while (true) {
                    const { done, value } = await reader.read();

                    if (done) {
                        console.log("SSE stream ended, reconnecting...");
                        break;
                    }

                    buffer += decoder.decode(value, { stream: true });

                    const lines = buffer.split('\n\n');
                    buffer = lines.pop() || '';

                    for (const line of lines) {
                        if (line.trim()) {
                            const dataMatch = line.match(/^data: (.+)$/m);
                            if (dataMatch) {
                                const data = dataMatch[1];
                                console.log('Forwarding:', data);
                                const { error } = emit('message', data);

                                if (error) {
                                    console.log('Client disconnected, cleaning up...');
                                    reader.cancel();
                                    return; // Exit completely - client is gone
                                }
                            }
                        }
                    }
                }
            } catch (error) {
                console.error("Error in SSE stream:", error);

                // Clean up reader if it exists
                if (reader) {
                    try {
                        await reader.cancel();
                    } catch (e) {
                        // Ignore cancel errors
                    }
                }

                await delay(5000);
            }
        }
    })
}

/**
 * @param {number} milliseconds
 */
function delay(milliseconds) {
    return new Promise(resolve => setTimeout(resolve, milliseconds));
}
