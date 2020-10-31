module Toolbar exposing ( ToolbarInnerMsg(..)
                        , ToolbarModel
                        , init
                        , update
                        , toolbar
                        , translate)

import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Theme.Toolbar as Theme
import Theme.Icons as Icons
import Element.Font
import Html.Attributes exposing (title)
import Element.Input
import Element.Input exposing (labelRight)
import Element.Input exposing (labelHidden)
import Html.Events
import Json.Decode as De
import Element.Input
import Animator.Css exposing (backgroundColor)
import Element.Input exposing (OptionState(..))
import Element.Input exposing (optionWith)
import Theme.Body

{-| Model 

    ToolbarModel 
        searchText: text in searchBox
        title: Title in toolbar
        subTitle: subtitle under title
        navbarButton: navbar button toggle status

-}

type alias ToolbarOption msg =
    { label : String
    , onSelected : msg
    }

type alias OptionList msg = 
    { selected : Maybe msg 
    , list : List ( ToolbarOption msg )
    }

type alias ToolbarModel msg =
    { searchText : String
    , title : String
    , subTitle : String
    , navbarButton : Bool
    , options : OptionList msg
    }

asSelectedOption : OptionList msg -> msg -> OptionList msg
asSelectedOption optionList option = 
    { optionList | selected = Just option }

asOptionList : ToolbarModel msg -> OptionList msg -> ToolbarModel msg
asOptionList model list = 
    { model | options = list }

init : List ( ToolbarOption msg ) -> Bool -> String -> Maybe String -> ToolbarModel msg
init list status title subTitle = 
    ToolbarModel 
    ""  
    title 
    (Maybe.withDefault "desde la decada 1970" subTitle) 
    status
    { selected = Nothing
    , list = list }

{-| ( Msg msg ): ( Msg msg ) emitted by this view
        Outer( Msg msg ) are to be managed by parent view
        Inner( Msg msg ) are to be managed by this view
        
    ToolbarOuter( Msg msg ) are messages sent to parent view
        SearchIssued are sent when the search box is submitted,
            carries the text issued  

        NavbarToggled navbar button was pressed by user

    ToolbarInner( Msg msg ) are messages handled by this view
        SearchTextChange message on search box changed by user input
        NavbarEnvToggled navbar was toggled by an external actor
        StatusUpdated title and subtitled are changed, usually 
            by an external actor

-}

type ToolbarInnerMsg msg
    = SearchTextChanged String
    | NavbarEnvToggled
    | StatusUpdated String String
    | OptionEnvSelected msg

type ToolbarOuterMsg msg
    = SearchIssued String
    | NavbarToggled
    | OptionSelected msg

type Msg msg 
    = InnerMsg ( ToolbarInnerMsg msg )
    | OuterMsg ( ToolbarOuterMsg msg )

update : ToolbarInnerMsg msg -> ToolbarModel msg ->  ( ToolbarModel msg, Cmd ( ToolbarInnerMsg msg ) )
update msg model =
    case msg of
        SearchTextChanged newText ->
             ( { model | searchText = newText }, Cmd.none )

        NavbarEnvToggled ->
             ( { model | navbarButton = not model.navbarButton }, Cmd.none )

        StatusUpdated newTitle newSubTitle ->
            ( { model | title = newTitle
                      , subTitle = newSubTitle }, Cmd.none )

        OptionEnvSelected option -> 
            (
                option
                |> asSelectedOption model.options
                |> asOptionList model 
                , Cmd.none
            )

{-| Helper view functions

    Helper functions to divide rendering into components

        searchBar to render the search box

        titleAndSubtitle to render the title and subtitle displayed

-}

searchBar : ToolbarModel msg -> Element ( Msg msg )
searchBar { searchText } = 
        Element.Input.text [ Border.rounded 10
                           , htmlAttribute <| Html.Events.on "keydown" ( onKeyDown <| OuterMsg <| SearchIssued searchText )
                           , height fill
                           , width shrink
                           , Element.Font.size 18
                           , alignRight ]
                           { label = labelHidden "whatever"
                           , placeholder = Just
                                          <| Element.Input.placeholder 
                                              [ padding 0
                                              , width shrink ]
                                              <| row [ alignLeft 
                                                     , alignTop 
                                                     , spacing 10
                                                     , centerY
                                                     , height shrink ]
                                                     [ html <| Icons.search 
                                                     , el [paddingEach { left = 10, right = 0, top = 0, bottom = 0} ] <| text "Escriba el año..."]
                            , text = searchText
                            , onChange = SearchTextChanged >> InnerMsg }

titleAndSubtitle : ToolbarModel msg -> Element a 
titleAndSubtitle { title, subTitle }= 
    column [ alignLeft 
           , height fill ]
           [ el [ Element.Font.semiBold 
                , alignTop
                , Element.Font.color Theme.titleFont
                , Element.Font.size 22 ] <| text title
           , el [ Element.Font.light 
                , Element.Font.color Theme.subtitleFont
                , Element.Font.italic
                , Element.Font.size 16 ] <| text subTitle ]

customOption : Int -> String -> OptionState -> Element msg
customOption index title value = 
    let
        backgroundColor = case value of 
                            Idle  -> Theme.mainColor
                            Focused -> Theme.mainColor
                            Selected -> Theme.Body.background                         

        negated = (1 - index)

        borderStyle = Border.roundEach { topRight = 3*index
                                       , bottomRight = 3*index
                                       , topLeft = 3*negated
                                       , bottomLeft = 3*negated }

        borderWidth = Border.widthEach { top = 1
                                       , bottom = 1
                                       , right = index
                                       , left = 1 }
    in
    el 
    [ Background.color backgroundColor
    , borderWidth
    , Element.Font.medium
    , Element.Font.size 16
    , Border.color <| rgb 0 0 0
    , Border.solid
    , if value == Selected then
        Border.glow ( Theme.searchFont ) 2
      else
        Border.glow ( rgb 0 0 0 ) 0
    , borderStyle ]
    <| el [ padding 5 ] <| text title

toolbarOptions : ToolbarModel msg -> Element ( Msg msg )
toolbarOptions { options } = 
    Element.Input.radioRow 
    [ Border.rounded 5 ]
    { onChange = OuterMsg << OptionSelected 
    , selected = options.selected 
    , label = labelHidden "options"
    , options = List.indexedMap
                (\index option -> optionWith option.onSelected ( customOption index option.label )) 
                options.list }

navBarButton : ToolbarModel msg -> Element ( Msg msg )
navBarButton { navbarButton } = 
    Element.Input.button [ height fill
                         , centerY
                         , padding 5
                         , width shrink ]
                         { label = html <| Icons.menu
                         , onPress = Just <| OuterMsg <| NavbarToggled }

{-| View
    Renders the toolbar
-}

toolbar : ToolbarModel msg -> Element ( Msg msg )
toolbar model =
    row [ width fill
        , scrollbarX
        , Background.color Theme.mainColor
        , Border.color Theme.borderColor
        , Border.widthEach { top = 0, bottom = 2, right = 0, left = 0}
        , padding 15
        , spacing 15
        , Border.solid
        , height <| px 70]
    [ navBarButton model 
    , titleAndSubtitle model
    , searchBar model 
    , el [ width shrink, alignRight ] <| toolbarOptions model
    ]

{-| Translator 
    Exposes the messages sent by this component
    the exposed ones are the same as ToolbarOuter( Msg msg )
-}

translate:  { onInternal: ToolbarInnerMsg b -> msg, onNavbarToggled: msg, onSearchIssued: String -> msg, onOptionSelected : b -> msg  } -> ( Msg b ) -> msg
translate { onInternal, onNavbarToggled, onSearchIssued, onOptionSelected } msg = 
    case msg of
        InnerMsg m -> onInternal m
        OuterMsg m -> 
            case m of
                NavbarToggled -> onNavbarToggled
                SearchIssued text -> onSearchIssued text
                OptionSelected b -> onOptionSelected b

{-| onKeyDown
    On key down helper decoder to detect when a Key is pressed
-}

onKeyDown: msg -> De.Decoder msg
onKeyDown msg = 
    Html.Events.keyCode 
    |> De.andThen (\key -> 
                    if key == 13 then
                        De.succeed msg
                    else
                        De.fail "Enter was not pressed" )