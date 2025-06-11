# [imagine.willow.sh](https://imagine.willow.sh)

Dynamically generate and cache spongebob imagine gifs on the edge (blazingly fast ⚡️)

The text you want to display is the filename of the gif. It supports percent encoded URLs, additionally treating and `_`s as spaces. It'll attempt to clean your input, and redirect you to the updated URL if it does. Emojis currently don't work yet, and line wrapping is not supported.

For example:

```
https://imagine.willow.sh/imagine.gif
```

![spongebob rainbow gif saying "imagine"](https://imagine.willow.sh/imagine.gif)


<details>
  <summary>More Examples</summary>

  ### Spaces are underscores

  ```
  https://imagine.willow.sh/spaces_are_underscores.gif
  ```

  ![spongebob rainbow gif saying "spaces are underscores"](https://imagine.willow.sh/spaces_are_underscores.gif)

  ### Percent Encoding

  ```
  https://imagine.willow.sh/percent_encoding_%3A).gif
  ```

  ![spongebob rainbow gif saying "percent encoding :)"](https://imagine.willow.sh/percent_encoding_%3A\).gif)

  ### Emojis

  (not working currently)

  ```
  https://imagine.willow.sh/%F0%9F%A7%BD.gif
  ```

  ![spongebob rainbow gif saying "🧽"](https://imagine.willow.sh/%F0%9F%A7%BD.gif)
</details>
