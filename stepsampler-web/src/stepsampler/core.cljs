(ns stepsampler.core
  (:require [stepsampler.ui :as ui]
            [replicant.dom :as r]))

(defn main []
  (.addEventListener js/document.body "dragenter" (fn [event] (.preventDefault event)))
  (.addEventListener js/document.body "dragover" (fn [event] (.preventDefault event)))
  (.addEventListener js/document.body "drop"
  (fn [e]
    (.preventDefault e) ;; VERY IMPORTANT
    (println "drop event fired" e)
    (let [files (.. e -dataTransfer -files)]
      (doseq [file (array-seq files)]
        (println "Dropped file:" (.-name file))))))

  (let [element (js/document.getElementById "app")]
    (when element
      (r/render element (ui/app-screen 0)))))
