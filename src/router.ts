import { createRouter, createWebHashHistory } from "vue-router";
import HomeView from "./views/HomeView.vue";
import ColumnView from "./views/ColumnView.vue";
import ArticleView from "./views/ArticleView.vue";
import SettingsView from "./views/SettingsView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: HomeView },
    { path: "/c/:slug", name: "column", component: ColumnView, props: true },
    {
      path: "/c/:slug/:filename",
      name: "article",
      component: ArticleView,
      props: true,
    },
    {
      path: "/settings/:section?",
      name: "settings",
      component: SettingsView,
    },
    { path: "/admin", redirect: "/settings/path" },
  ],
  scrollBehavior() {
    return { top: 0 };
  },
});
