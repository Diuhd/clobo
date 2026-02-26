<script lang="ts">
  import {listen} from "@tauri-apps/api/event";
  import {onMount} from "svelte";
  import {commands, type PreviewData} from "../bindings";
  import ClipboardButton from "./ClipboardButton.svelte";
  import {Tabs} from '@skeletonlabs/skeleton-svelte';

  let preview_list: PreviewData[] = $state([]);
  const inflight = new Map<number, number>(); // id -> token

  onMount(() => {
    commands.loadDataPreviewList(1).then(data => {
      if (data.status === "ok") {
        preview_list = data.data;
      } else {
        console.log(data.error);
      }
    });

    listen<number>('clipboard_update', async (event) => {
        const id = event.payload;
        const token = (inflight.get(id) ?? 0) + 1;
        inflight.set(id, token);

        const idx = preview_list.findIndex((x) => x.id === id);
        if (idx !== -1) preview_list.splice(idx, 1);

        const res = await commands.loadPreview(id);
        if (res.status !== "ok") return;

        if (inflight.get(id) !== token) return;

        preview_list = [res.data, ...preview_list];
    });
  })
</script>

<main class="container">
    <div class="p-1.5">
        <Tabs defaultValue="clipboard">
            <Tabs.List>
                <Tabs.Trigger class="flex-1" value="clipboard">Clipboard</Tabs.Trigger>
                <Tabs.Trigger class="flex-1" value="assistant">Assistant</Tabs.Trigger>
                <Tabs.Indicator/>
            </Tabs.List>
            <Tabs.Content value="clipboard">
                <div id="clipboard_div">
                    {#each preview_list as elem}
                        <div class="flex justify-center pb-2">
                            <ClipboardButton preview={elem.preview} id={elem.id}></ClipboardButton>
                        </div>
                    {/each}
                </div>
            </Tabs.Content>
        </Tabs>
    </div>
</main>