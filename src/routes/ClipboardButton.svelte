<script lang="ts">
    import {Menu as IconMenu, Star, Trash2, Brain} from '@lucide/svelte';
    import {Menu, Portal} from '@skeletonlabs/skeleton-svelte'
    import {type ClipboardData, commands} from "../bindings";
    import {toaster} from "./toaster";

    let { preview, id } = $props();

    let pinned = $state(false);

    function onclick() {
        commands.loadData(id).then(data => {
            if (data.status == 'ok') {
                let full_data: ClipboardData = data.data;
                navigator.clipboard.writeText(full_data.content);
                toaster.success({
                    title: 'Success',
                    description: 'Copied item to clipboard!'
                });
                // TODO: show toast "Copied to Clipboard"
            } else {
                console.log(data.error);
            }
        })
    }
</script>

<div
        id="clipboard_button"
        class="relative inline-block group"
        role="group"
        aria-label="Clipboard Action Group"
>
    <button
            type="button"
            class="bg-primary-500 text-primary-contrast-500 text-[15px] rounded-[3px]
               outline-2 outline-transparent wbutton flex items-start text-left h-24 group-hover:outline-surface-100
               transition-all duration-200 overflow-hidden"
            {onclick}
    >
        <span class="pt-3 pl-3 pb-3 line-clamp-3 whitespace-pre-wrap overflow-hidden w-78">
            {preview}
        </span>
    </button>

    <Menu>
        <Menu.Trigger class="btn-icon preset-filled-primary-500 absolute top-2 overflow-hidden right-2 w-5 h-5"><IconMenu size={18} /></Menu.Trigger>
        <Portal>
            <Menu.Positioner>
                <Menu.Content>
                    <Menu.Item value="delete">
                        <Menu.ItemText><Trash2 size={18} /></Menu.ItemText>
                        <Menu.ItemText>Delete</Menu.ItemText>
                    </Menu.Item>
                    <Menu.Item value="assistant">
                        <Menu.ItemText><Brain size={18} /></Menu.ItemText>
                        <Menu.ItemText>Assistant</Menu.ItemText>
                    </Menu.Item>
                </Menu.Content>
            </Menu.Positioner>
        </Portal>
    </Menu>

    <button
            id="pin"
            type="button"
            class="btn-icon preset-filled-primary-500 absolute bottom-2 right-2 w-5 h-5"
            onclick={(e) => { e.stopPropagation(); console.log('pin'); }}
    >
        <Star size={18}></Star>
    </button>
</div>

<style>
    button {
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
        text-rendering: optimizeLegibility;
    }
</style>