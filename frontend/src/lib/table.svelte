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

<style>
table {
    width: 100%;
    border-collapse: collapse;
}

th {
    text-align: left;
    padding: 4px;
    font-weight: bold;
}

td {
    text-align: right;
    padding: 4px;
}
</style>