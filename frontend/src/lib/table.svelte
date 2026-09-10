<table>
    <tbody>
        {#each tableColumns as column}
            <tr>
                <th>{column.header}</th>
                <td>{column.value}</td>
            </tr>
        {/each}
    </tbody>
</table>

<script lang="ts">
    import type { TableColumn } from "./types";

    let { tableColumns }: { tableColumns: TableColumn[] } = $props();
</script>