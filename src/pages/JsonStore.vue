<template>
    <q-page padding>
        <div class="q-pa-md">
            <h3>JSON Store</h3>
            <div class="q-mt-md">
                <q-btn color="primary" @click="fetchItems">
                    Fetch Items
                </q-btn>
            </div>
            <div class="q-mt-md">
                <q-input v-model="newItem" label="New Item" outlined />
                <q-btn class="q-mt-sm" color="primary" @click="addItem(newItem)">
                    Add Item
                </q-btn>
            </div>

            <div class="q-mt-md">
                <q-list bordered>
                    <q-item-label header>Items</q-item-label>
                    <q-item v-for="(item, index) in items" :key="index">
                        <q-item-section>
                            {{ item }}
                        </q-item-section>
                        <q-item-section side>
                            <q-btn color="negative" size="sm" @click="deleteItem(index)">
                                Delete
                            </q-btn>
                        </q-item-section>
                    </q-item>
                </q-list>
            </div>
        </div>
    </q-page>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from "@tauri-apps/api/core";

const items = ref<string[]>([]);
const newItem = ref<string>("");

async function fetchItems() {
    items.value = await invoke("list_items");
}

async function addItem(value:string) {
    invoke<string[]>('add_item', { value })
        .then(updatedItems => {
            items.value = updatedItems;
            newItem.value = "";
        });
}

async function deleteItem(index: number) {
    invoke<string[]>('delete_item', { index })
        .then(updatedItems => {
            items.value = updatedItems;
        });
}

fetchItems();

</script>