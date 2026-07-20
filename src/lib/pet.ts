import { invoke } from "@tauri-apps/api/core";

export type PetCorner = "topLeft" | "topRight" | "bottomLeft" | "bottomRight";

/** Typed wrappers for the desktop companion overlay window. */
export const pet = {
  show: () => invoke<void>("show_pet_overlay"),
  hide: () => invoke<void>("hide_pet_overlay"),
  teleport: (corner: PetCorner) => invoke<void>("teleport_pet", { corner }),
};
