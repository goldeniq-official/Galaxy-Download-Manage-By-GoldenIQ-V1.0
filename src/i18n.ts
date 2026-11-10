// SPDX-License-Identifier: RPL-1.5
// Khmer first (R6); every visible string comes from these files (P1 #31: no hard-coded UI text).
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import km from "./locales/km.json";
import en from "./locales/en.json";

void i18n.use(initReactI18next).init({
  resources: { km: { translation: km }, en: { translation: en } },
  lng: "km",
  fallbackLng: "en",
  interpolation: { escapeValue: false }, // React renders text nodes, never HTML (X-01)
});

export default i18n;
