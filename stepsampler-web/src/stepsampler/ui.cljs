(ns stepsampler.ui)

(defn drop-files-area [dropped-n]
  [:div.dropzone {:id "drop-files-area"}
   [:div.content {:aria-hidden "true"}
    [:svg.icon {:viewBox "0 0 24 24"
                :fill "none"
                :xmlns "http://www.w3.org/2000/svg"}
     [:path {:d "m8 8 4-4 4 4"
             :stroke "#000000"
             :stroke-width "1.5"
             :stroke-linecap "round"
             :stroke-linejoin "round"}]
     [:path {:d "M12 4v12M19 17v.6c0 1.33-1.07 2.4-2.4 2.4H7.4C6.07 20 5 18.93 5 17.6V17"
             :stroke "#000000"
             :stroke-width "1.5"
             :stroke-miterlimit "10"
             :stroke-linecap "round"}]]
    [:h1 "Drop WAV samples at the page"]

    [:button.start-btn {:type "button" :disabled (= dropped-n 0)}
     "Start"]
     
    [:p.muted-text 
     (if (= dropped-n 0)
       "No files selected"
       (str dropped-n " file" (when (not= dropped-n 1) "s") " selected"))]
    ]])

(defn help-screen []
  [:div.help
   [:h2 "What is this app about?"]
   [:p  "This is Stepsampler: it makes a sample by joining together several evenly lengthed samples."]
   [:p " This approach often
    used in hardware samplers allows many sounds to be packed into one slot and triggered
    through chopping/slicing. The program was designed with the Roland P-6 in mind where
    such percussion sample packs are especially useful -- for example fitting dozens of
    hi-hats onto a single pad to free up space for actual music projects."]
   [:p "This app requries WASM support in your browser."]
   [:ol
    [:li "Drop several WAV samples at the page."]
    [:li "The app will truncate silence at the beginning and end of the samples,"]
    [:li "normalize by volume,"]
    [:li "convert to 44.1k 16bit mono,"]
    [:li "append silence to have each sample to have the same length"]
    [:li "concatenate into a single sample."]]])

(defn app-screen [dropped-n]
  [:div.layout
   [:aside (help-screen)]
   [:main (drop-files-area dropped-n)]])