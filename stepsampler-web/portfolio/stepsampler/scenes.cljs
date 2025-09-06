(ns stepsampler.scenes
  (:require [portfolio.replicant :refer-macros [defscene]]
            [portfolio.ui :as portfolio]
            [stepsampler.ui :as ui]))

(defscene help-screen
  (ui/help-screen))

(defscene drop-files-area-disabled
  (ui/drop-files-area 0))
  
(defscene drop-files-area-enabled 
  (ui/drop-files-area 5))

(defscene app
  (ui/app-screen 5))

(defn main []
  (portfolio/start!
   {:config
    {:css-paths ["/styles.css"]
     :viewport/defaults
     {:background/background-color "#fdeddd"}}}))
