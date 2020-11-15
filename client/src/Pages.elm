module Pages exposing (..)
import Element exposing (..)
import Element.Input
import Html.Attributes exposing (name)
import Element.Input exposing (labelHidden)

type PageMsg = 
    TextChanged String
    | InsertIssued 
    | SearchIssued

type alias PageModel =
    { id: Int
    , name: String
    }

init: PageModel 
init =
    {
        id = 0,
        name = "Holimundo"
    }

update : PageMsg -> PageModel ->  ( PageModel, Cmd PageMsg )
update msg model =
    case msg of
        TextChanged _ ->
            ( model, Cmd.none )
        InsertIssued ->
            ( model, Cmd.none )
        SearchIssued -> 
            ( model, Cmd.none )

view: PageModel -> Element PageMsg
view { id, name } = 
    row [ width fill 
        , height fill ] 
        [
            column [ width fill ]
                   [ text "ID actual" 
                   , String.fromInt id |> text ],
            column [ width fill ]
                   [ text "Nombre a Insertar" 
                   , Element.Input.text [] { onChange = TextChanged
                                           , text = name
                                           , placeholder = Nothing
                                           , label = labelHidden "hoi"} ],
            column [ width fill ]
                   [ Element.Input.button [] { onPress = Just InsertIssued
                                             , label = text "Insertar nuevo registro" } ],
            column [ width fill ]
                   [ Element.Input.button [] { onPress = Just SearchIssued
                                             , label = text "Buscar registro con ID" } ] 
        ]