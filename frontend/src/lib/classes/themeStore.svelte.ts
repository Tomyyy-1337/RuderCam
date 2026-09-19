import { writable } from 'svelte/store';

export type Theme = 'light' | 'dark'

export function setTheme(newTheme: Theme): void {
    if (newTheme === 'light') {
        document.documentElement.style.setProperty('--background', '#d3d3d3');
        document.documentElement.style.setProperty('--section-background', '#b7b7b7');
        document.documentElement.style.setProperty('--text', '#000000');
    } else {
        document.documentElement.style.setProperty('--background', '#0f0f0f');
        document.documentElement.style.setProperty('--section-background', '#1b1b1b');
        document.documentElement.style.setProperty('--text', '#ffffff');
    }

    localStorage.setItem('theme', newTheme);
    currentTheme.set(newTheme);
}

export const currentTheme = writable<Theme>("light");