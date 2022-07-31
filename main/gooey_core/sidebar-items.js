initSidebarItems({"constant":[["PRIMARY_WIDGET_CLASS","The name of the class assigned to widgets that have a solid background but should be colored with the primary UI color."],["ROOT_CLASS","The name of the class assigned to the root widget of a window."],["SOLID_WIDGET_CLASS","The name of the class assigned to widgets that have a solid background"]],"enum":[["LocalizationParameter","A parameter used in localization."]],"macro":[["rgb","Creates a [`Color`] using values ranging from 0-255. Accepts 3 parameters for an opaque RGB color, or 4 parameters for an RGBA color."]],"mod":[["assets","Types used for handling assets."],["styles","Types used for styling."]],"struct":[["AnyTransmogrifierContext","A context used internally when the [`Transmogrifier`] type cannot be known."],["AppContext","A context used during initialization of a window or application."],["Callback","A callback that receives information `I`, and returns `R`."],["Channels","Communication channels used to communicate between [`Widget`]s and `Transmogrifier`s."],["Context","Enables [`Widget`]s to send commands to the `Transmogrifier`."],["Gooey","A graphical user interface."],["LocalizationParameters","Parameters used in localization strings."],["LockedWidget","A locked widget reference. No other threads can operate on the widget while this value is alive."],["ManagedCodeGuard","A guard marking that Gooey-managed code is executing."],["StyledWidget","A widget and its initial style information."],["Timer","Invokes a [`Callback`] after a delay."],["TransmogrifierContext","A context passed into [`Transmogrifier`] functions with access to useful data and types. This type is mostly used to avoid passing so many parameters across all functions."],["TransmogrifierState","Generic storage for a transmogrifier."],["Transmogrifiers","A collection of transmogrifiers to use inside of a frontend."],["UnscheduledTimer","A [`Timer`] that hasn’t been scheduled."],["WeakTimer","A weak reference to a [`Timer`]. Uses [`Arc`] and [`Weak`] under the hood."],["WeakWidgetRegistration","References an initialized widget. These references will not keep a widget from being removed."],["WidgetGuard","A reference to a locked widget."],["WidgetId","A unique ID of a widget, with information about the widget type."],["WidgetRef","A widget reference. Does not prevent a widget from being destroyed if removed from an interface."],["WidgetRegistration","References an initialized widget. On drop, frees the storage and id."],["WidgetState","Generic, clone-able storage for a widget’s transmogrifier."],["WidgetStorage","Generic-type-less widget storage."],["WindowBuilder","A builder for a Window."],["WindowConfiguration","Configuration options used when opening a window."]],"trait":[["AnyChannels","A generic-type-less trait for [`Channels`]"],["AnyFrontend","An interface for Frontend that doesn’t requier knowledge of associated types."],["AnySendSync","A value that can be used as [`Any`] that is threadsafe."],["AnyTransmogrifier","A Transmogrifier without any associated types."],["AnyWidget","A Widget without any associated types. Useful for implementing frontends."],["AnyWindowBuilder","A [`WindowBuilder`] that has had its widget type parameter erased."],["CallbackFn","A callback implementation. Not typically directly implemented, as this trait is auto-implemented for any `Fn(I) -> R` types."],["DefaultWidget","A widget that can be created with defaults."],["Frontend","A frontend is an implementation of widgets and layouts."],["Key","A key for a widget."],["KeyedStorage","A type that registers widgets with an associated key."],["Localizer","A type that provides localization (multi-lingual representations of text)."],["NativeTimer","A native timer implementation."],["RelatedStorage","Related storage enables a widget to communicate in a limited way about widgets being inserted or removed."],["Transmogrifier","Transforms a Widget into whatever is needed for [`Frontend`] `F`."],["Widget","A graphical user interface element."],["Window","Represents a window."]],"type":[["Pixels","A unit representing physical pixels on a display."],["Scaled","A unit aiming to represent the scaled resolution of the display the interface is being displayed on. The ratio between [`Pixels`] and `Scaled` can vary based on many things, including the display configuration, the system user interface settings, and the browser’s zoom level. Each [`Frontend`] will use its best available methods for translating `Scaled` to [`Pixels`] in a way that is consistent with other applications."],["WindowRef","A clonable reference to a window."]]});