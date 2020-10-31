module Navbar exposing (NavInternalMsg(..), NavOuterMsg, PageInfo, PageList, NavModel, NavbarState, translate, update, sidebar, init, subscriptions)
import Element exposing (..)
import Animator exposing (..)
import Element.Background as Background
import Element.Events as Events
import Theme.Sidebar as Theme
import Theme.Icons as Icon
import Element.Font
import Element.Border
import Html exposing (Html)
import Theme.Body exposing (background)
import Time

{-| PageList

    Represents the page list shown on the navbar and it's
    selected page
-}
type alias PageList msg =
    {
    selectedPage: Maybe Int
    , pages: List (PageInfo msg)
    }

{-| PageInfo

    Represents the information shown on the navbar about the page,
    and the msg it sends when selected
-}
type alias PageInfo msg =
    {
    label: String
    , icon: Html (NavMsg msg)
    , pageInfo: msg
    }

{-| Hover

    Represents what button on the navbar is hovered or if none is
-}
type Hover = 
    Simul
    | Back
    | Settings
    | None

{-| NavbarState

    Represents if the navbar is collapsed or expanded
-}
type NavbarState = 
    Opened
    | Closed

{-| NavModel

    Represents the model the Navbar logic is meant to handle
-}
type alias NavModel msg =
    {
    pageList: PageList msg
    , navState: Animator.Timeline NavbarState
    , hovered: Hover
    }

init: List ( PageInfo msg ) -> NavModel msg
init list = 
    NavModel (PageList Nothing list) (Animator.init Opened) None

animator: Animator.Animator (NavModel msg)
animator = 
    Animator.animator
    |> Animator.watching
        .navState
        (\newState model ->
            { model | navState = newState } )

asSelectedPage: PageList msg -> Int -> PageList msg
asSelectedPage list index = 
    { list | selectedPage = Just index }

asPageList: NavModel msg -> PageList msg -> NavModel msg
asPageList model list = 
    { model | pageList = list }

type NavInternalMsg = 
    Hovered Hover
    | StateToggled
    | PageChanged Int
    | Tick Time.Posix
    | SimulStarted
                    
type NavOuterMsg msg = 
    PageSelected (PageInfo msg) Int
    | SimulInit

type NavMsg msg = 
    OutMsg (NavOuterMsg msg)
    | InnerMsg NavInternalMsg

clickableEl: List (Attribute msg) -> Element msg -> Element msg
clickableEl list =
    el <| pointer :: list

simulBackground: NavModel msg -> Element (NavMsg mgs)
simulBackground { hovered }=
    let
        color = (if hovered == Simul |> not then "#00a8ff" else "#4cd137")
    in
        image [ inFront <|
                    column [ width fill
                           , Background.color Theme.simulBgColor
                           , height fill
                           , spacing 10
                           ] [ clickableEl [ centerX
                                           , centerY
                                           , Events.onClick <| OutMsg <| SimulInit
                                           , Events.onMouseLeave <| InnerMsg <| Hovered None 
                                           , Events.onMouseEnter <| InnerMsg <| Hovered Simul ]
                                   <| html <| Icon.playCircle color
                             , clickableEl [ Element.Font.center
                                           , Element.Font.color Theme.fontColor
                                           , Element.Font.medium
                                           , Element.Font.shadow
                                                 { offset = ( 2, 2 ), blur = 1, color = rgb 0 0 0 }
                                           , Element.Font.size 24
                                           , centerY
                                           , centerX ]
                                   <| text "Iniciar simulación"]
              , width fill
              , height <| minimum 200 <| shrink
              , centerX ]
        { src = "https://cdn-1.motorsport.com/images/vmt/10Rd1RZY/s1/fia-wec-6-hours-of-cota-hou-1.jpg", description = "FIA WEC image" }
        

commandBox: NavModel msg -> Element (NavMsg msg)
commandBox { hovered }=
    let
        backbColor =  if hovered == Back |> not then "#00a8ff" else "#4cd137"
        settingsColor =  if hovered == Settings |> not then "#00a8ff" else "#4cd137"
    in
        row [ width fill
            , Background.color Theme.mainColor
            , padding 5
            ]
        [
         clickableEl [
              alignLeft
             , Events.onClick <| InnerMsg <| StateToggled
             , Events.onMouseEnter <| InnerMsg <| Hovered Back
             , Events.onMouseLeave <| InnerMsg <| Hovered None
             ] <| html <| Icon.arrowLeftCircle backbColor
       , clickableEl [
               alignRight
             , Events.onMouseEnter <| InnerMsg <| Hovered Settings
             , Events.onMouseLeave <| InnerMsg <| Hovered None
             ] <| html <| Icon.settings settingsColor
        ]

option : Maybe Int -> Int -> PageInfo msg -> Element (NavMsg msg)
option selected index pageInfo =
    let
        bgColor = selected
                  |> Maybe.andThen (\number -> if number == index then
                                                   Just number
                                               else
                                                   Nothing )
                  |> Maybe.map (\_ -> Theme.focusColor)
                  |> Maybe.withDefault Theme.mainColor
    in
        row [ width fill
            , padding 20
            , spacing 20
            , mouseOver [ Background.color Theme.focusColor ]
            , Background.color bgColor
            , pointer
            , Events.onClick <| OutMsg <| PageSelected pageInfo index ]
        [ el [ centerY
             , alignLeft ]
              <| html pageInfo.icon
        , el [ centerY
             , alignLeft
             , Element.Font.color Theme.fontColor ]
              <| text pageInfo.label ]

options : PageList msg -> Element (NavMsg msg)
options { selectedPage, pages } =
    column [ width fill
           , height fill
           , Background.color Theme.mainColor ]
    (List.indexedMap (option selectedPage) pages)

sidebar:  List ( Attribute (NavMsg msg) ) -> NavModel msg -> Element (NavMsg msg)
sidebar list model =
    column ([height fill 
           , width <| px <| round <| Animator.linear model.navState (\state -> if state == Opened then Animator.at 300 else Animator.at 0)
           , alpha <| Animator.linear model.navState (\state -> if state == Opened then Animator.at 1 else Animator.at 0) ] ++ list)
        [ commandBox model
        , simulBackground model
        , options model.pageList ]

update: NavModel msg -> NavInternalMsg -> ( NavModel msg, Cmd NavInternalMsg )
update model imsg =
    let
        state = (if Animator.current model.navState == Opened then Closed else Opened)
    in case imsg of
        Tick posix -> ( Animator.update posix animator model, Cmd.none )
        Hovered hover -> ({ model | hovered = hover }, Cmd.none)
        StateToggled -> ({ model | navState = Animator.go veryQuickly state model.navState }, Cmd.none)
        PageChanged index -> ( index
                               |> asSelectedPage model.pageList 
                               |> asPageList model, Cmd.none)
        SimulStarted -> ( model, Cmd.none )

translate: { onInternal: NavInternalMsg -> omsg, onSimul: omsg, onPage: msg -> Int -> omsg } -> (NavMsg msg) -> omsg
translate { onInternal, onSimul, onPage } msg =
    case msg of
        OutMsg omsg ->
            case omsg of
                SimulInit -> onSimul
                PageSelected { pageInfo } index -> onPage pageInfo index
        InnerMsg omsg -> onInternal omsg

subscriptions : NavModel msg -> Sub NavInternalMsg
subscriptions model =
    Animator.toSubscription Tick model animator