<script lang="ts">
    import {createClient, FetchTransport} from "@rspc/client"
    import type {Procedures} from "../../../back/bind"

    const client = createClient<Procedures>({
        transport: new FetchTransport("http://localhost:3000/rspc"),
    })
</script>
<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
{#await client.query(["version"])}
    <p>loading</p>
{:then res}
    <p>{res}</p>
{:catch error}
    <p>{error}</p>
{/await}
